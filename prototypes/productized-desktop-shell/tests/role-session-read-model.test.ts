import {
  createRoleSessionContinuationStartRequest,
  createRoleSessionDetailRequest,
  createRoleSessionDirectoryRequest,
  createRoleSessionReadEpoch,
  mergeRoleSessionDirectoryPage,
  normalizeRoleSessionReadError,
  parseRoleSessionDetail,
  parseRoleSessionDirectory,
  resolveRoleSessionDirectorySelection,
  roleSessionDetailMatchesCurrentSelection,
  roleSessionDirectoryPageHasCompatibleProjection,
  roleSessionDetailMatchesRequest,
  roleSessionDirectoryMatchesRequest,
  usableCurrentRoleSessionContinuationSelector,
  usableRoleSessionContinuationSelector,
} from "../src/lib/roleSessionReadModel";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

const labels = {
  role_label: "role:sha256:fixture",
  project_label: "scope:sha256:fixture",
  object_label: "object:sha256:fixture",
  channel_label: "channel:sha256:fixture",
  permission_label: "permission:sha256:fixture",
};

const directoryRequest = createRoleSessionDirectoryRequest({
  project_locator: "/m3c06/fixture/project",
  cursor: null,
  limit: 20,
  request_nonce: "directory-fixture-1",
});

const rawDirectory = {
  request_nonce: directoryRequest.request_nonce,
  projection_revision: "directory:fixture",
  entries: [
    {
      selection: "m3rs:opaque-selection",
      role_session_id: "session:sha256:fixture",
      session_revision: 3,
      labels,
      session_state: "ACTIVE",
      permission_state: "CURRENT",
      resolution_reason: null,
    },
  ],
  next_cursor: null,
};
const [rawOnlyDirectoryEntry] = rawDirectory.entries;
if (!rawOnlyDirectoryEntry) throw new Error("fixture raw directory must contain one server entry");

const directory = parseRoleSessionDirectory(rawDirectory);
assert(roleSessionDirectoryMatchesRequest(directory, directoryRequest), "目录回包 nonce 必须和当前请求匹配");
assert(directory.entries.length === 1, "目录保留服务端 RoleSession 条目");
const [onlyDirectoryEntry] = directory.entries;
if (!onlyDirectoryEntry) throw new Error("fixture directory must contain one server entry");
assertDeepEqual(
  resolveRoleSessionDirectorySelection(directory),
  { status: "automatic", selection: onlyDirectoryEntry.selection, rejected_selection: false },
  "只有完整且唯一的服务器目录条目才能自动选择",
);

const detailRequest = createRoleSessionDetailRequest({
  project_locator: directoryRequest.project_locator,
  selection: onlyDirectoryEntry.selection,
  request_nonce: "detail-fixture-1",
});
const rawDetail = {
  request_nonce: detailRequest.request_nonce,
  selection: detailRequest.selection,
  role_session_id: "session:sha256:fixture",
  session_revision: 3,
  projection_revision: "3:1:projection:fixture",
  labels,
  session_state: "ACTIVE",
  permission_state: "CURRENT",
  resolution_reason: null,
  context: {
    state: "AVAILABLE",
    retrieval_status: "COMPLETE",
    context_sources: ["source:sha256:fixture"],
    knowledge_refs: ["material:sha256:fixture", "skill:sha256:fixture"],
    gaps: [],
    source_links: [{ source_ref: "source:sha256:fixture", label: "source-label:sha256:fixture" }],
    request_more_material_available: false,
  },
  continuation: {
    state: "AVAILABLE",
    selector: "m3rs:opaque-continuation",
    reason: null,
  },
};
const detail = parseRoleSessionDetail(rawDetail);
assert(roleSessionDetailMatchesRequest(detail, detailRequest), "详情回包必须匹配选择和 nonce");
assert(
  usableRoleSessionContinuationSelector(detail) === "m3rs:opaque-continuation",
  "只有服务端 DTO 的 opaque selector 可以成为续聊目标",
);

const rawDirectoryB = {
  ...rawOnlyDirectoryEntry,
  selection: "m3rs:opaque-selection-b",
  role_session_id: "session:sha256:fixture-b",
  session_revision: 4,
  labels: {
    role_label: "role:sha256:fixture-b",
    project_label: "scope:sha256:fixture",
    object_label: "object:sha256:fixture-b",
    channel_label: "channel:sha256:fixture",
    permission_label: "permission:sha256:fixture",
  },
};
const multiDirectory = parseRoleSessionDirectory({
  ...rawDirectory,
  request_nonce: "directory-multi",
  entries: [rawOnlyDirectoryEntry, rawDirectoryB],
  next_cursor: null,
});
assertDeepEqual(
  resolveRoleSessionDirectorySelection(multiDirectory),
  { status: "selection_required", selection: null, rejected_selection: false },
  "同项目 A/B 目录不得按历史顺序暗中选择 A 或 B",
);
assertDeepEqual(
  resolveRoleSessionDirectorySelection(multiDirectory, "m3rs:unknown-selection"),
  { status: "selection_required", selection: null, rejected_selection: true },
  "未知 opaque selection 必须被当前服务器目录拒绝",
);
assertDeepEqual(
  resolveRoleSessionDirectorySelection(multiDirectory, rawDirectoryB.selection),
  { status: "explicit", selection: rawDirectoryB.selection, rejected_selection: false },
  "显式选择只能接受当前已加载的 B 条目",
);

const pagedSingleDirectory = parseRoleSessionDirectory({
  ...rawDirectory,
  request_nonce: "directory-paged-single",
  next_cursor: "m3rs:opaque-cursor",
});
assertDeepEqual(
  resolveRoleSessionDirectorySelection(pagedSingleDirectory),
  { status: "selection_required", selection: null, rejected_selection: false },
  "存在下一页时，即使当前页只有一条也必须等待显式选择",
);

const detailRequestB = createRoleSessionDetailRequest({
  project_locator: directoryRequest.project_locator,
  selection: rawDirectoryB.selection,
  request_nonce: "detail-fixture-b",
});
const detailB = parseRoleSessionDetail({
  ...rawDetail,
  request_nonce: detailRequestB.request_nonce,
  selection: detailRequestB.selection,
  role_session_id: rawDirectoryB.role_session_id,
  session_revision: rawDirectoryB.session_revision,
  projection_revision: "4:1:projection:fixture-b",
  labels: rawDirectoryB.labels,
  continuation: {
    state: "AVAILABLE",
    selector: "m3rs:opaque-continuation-b",
    reason: null,
  },
});
assert(
  !roleSessionDetailMatchesCurrentSelection(detail, detailRequest, multiDirectory, rawDirectoryB.selection),
  "迟到 A detail 不得覆盖已选择的 B",
);
assert(
  roleSessionDetailMatchesCurrentSelection(detailB, detailRequestB, multiDirectory, rawDirectoryB.selection),
  "选择 B 后只能消费当前 B detail",
);
assert(
  usableCurrentRoleSessionContinuationSelector(detailB, rawDirectoryB.selection, multiDirectory) === "m3rs:opaque-continuation-b"
    && usableCurrentRoleSessionContinuationSelector(detailB, onlyDirectoryEntry.selection, multiDirectory) === null,
  "continuation selector 只能跟随当前 B selection，不能回退到 A",
);

for (const [field, override] of [
  ["role_session_id", { role_session_id: "session:sha256:detail-drift" }],
  ["session_revision", { session_revision: 99 }],
  ["labels", { labels: { ...labels, permission_label: "permission:sha256:detail-drift" } }],
  ["session_state", { session_state: "CLOSED" }],
  ["permission_state", { permission_state: "REVALIDATION_REQUIRED" }],
  ["resolution_reason", { resolution_reason: "PERMISSION_REVALIDATION_REQUIRED" }],
] as const) {
  const inconsistent = parseRoleSessionDetail({ ...rawDetail, ...override });
  assert(
    !roleSessionDetailMatchesCurrentSelection(inconsistent, detailRequest, directory, onlyDirectoryEntry.selection),
    `同 selector 的 ${field} 漂移不得进入当前 detail`,
  );
  assert(
    usableCurrentRoleSessionContinuationSelector(inconsistent, onlyDirectoryEntry.selection, directory) === null,
    `同 selector 的 ${field} 漂移不得成为 continuation`,
  );
}

const pageWithDuplicateAndB = parseRoleSessionDirectory({
  request_nonce: "directory-page-2",
  projection_revision: "directory:page-two",
  entries: [rawOnlyDirectoryEntry, rawDirectoryB],
  next_cursor: null,
});
assert(
  roleSessionDirectoryPageHasCompatibleProjection(directory, pageWithDuplicateAndB, detail),
  "不同 revision 域的目录页与 detail 不应被字符串比较误判；重叠条目一致时可合并",
);
assertDeepEqual(
  mergeRoleSessionDirectoryPage(directory, pageWithDuplicateAndB).entries.map((entry) => entry.selection),
  [onlyDirectoryEntry.selection, rawDirectoryB.selection],
  "分页合并必须按服务器原有顺序去重，不从排序推断当前身份",
);
const pageWithDrift = parseRoleSessionDirectory({
  ...pageWithDuplicateAndB,
  request_nonce: "directory-page-drift",
  entries: [{ ...rawOnlyDirectoryEntry, session_revision: 99 }],
});
assert(
  !roleSessionDirectoryPageHasCompatibleProjection(directory, pageWithDrift, detail),
  "分页中可观察到的 session/projection 漂移必须 fail closed，不能保留旧 detail",
);

let unsafeDetailRejected = false;
try {
  parseRoleSessionDetail({
    ...rawDetail,
    owner_fingerprint: "must-not-cross-renderer-boundary",
    raw_transcript: "must-not-cross-renderer-boundary",
  });
} catch {
  unsafeDetailRejected = true;
}
assert(unsafeDetailRejected, "前端 DTO parser 必须 deny_unknown owner/raw transcript 字段");

// Frontend input is deny-unknown too. The object shape cannot smuggle role,
// owner, provider, profile, or thread truth into either fixed-host command.
for (const invalid of [
  { ...directoryRequest, role: "renderer-role" },
  { ...detailRequest, thread_id: "legacy-thread" },
  {
    project_locator: directoryRequest.project_locator,
    continuation_selector: "m3rs:opaque-continuation",
    request_nonce: "continuation-fixture-1",
    user_text: "fixture message",
    owner_fingerprint: "renderer-owner",
  },
]) {
  let rejected = false;
  try {
    if ("selection" in invalid) createRoleSessionDetailRequest(invalid as never);
    else if ("continuation_selector" in invalid) createRoleSessionContinuationStartRequest(invalid as never);
    else createRoleSessionDirectoryRequest(invalid as never);
  } catch {
    rejected = true;
  }
  assert(rejected, "renderer authority/thread truth 必须被 deny_unknown 拒绝");
}

const continuationPayload = createRoleSessionContinuationStartRequest({
  project_locator: directoryRequest.project_locator,
  continuation_selector: "m3rs:opaque-continuation",
  request_nonce: "continuation-fixture-1",
  user_text: "fixture message",
});
const payloadJson = JSON.stringify(continuationPayload);
for (const forbidden of ["actor", "role", "scope", "permission", "owner", "provider", "thread", "profile", "channel"]) {
  assert(!payloadJson.includes(forbidden), `continuation payload 不得包含 ${forbidden} truth`);
}

const epoch = createRoleSessionReadEpoch();
const oldEpoch = epoch.next();
const currentEpoch = epoch.next();
assert(!epoch.isCurrent(oldEpoch) && epoch.isCurrent(currentEpoch), "旧请求回包不能覆盖当前选择");
const directoryEffectEpoch = epoch.next();
const detailHandOffEpoch = epoch.next();
const cleanupEpoch = epoch.next();
assert(
  !epoch.isCurrent(directoryEffectEpoch)
    && !epoch.isCurrent(detailHandOffEpoch)
    && epoch.isCurrent(cleanupEpoch),
  "effect cleanup 必须废弃已接棒的 detail generation，不能只比较最初 directory epoch",
);
assert(
  !roleSessionDetailMatchesRequest(detail, { ...detailRequest, request_nonce: "stale-detail" }),
  "nonce 不匹配的详情回包必须被丢弃",
);

const disabled = parseRoleSessionDetail({
  ...rawDetail,
  continuation: { state: "DISABLED", selector: null, reason: "CONTEXT_GAPS_PRESENT" },
});
assert(usableRoleSessionContinuationSelector(disabled) === null, "缺资料或隔离状态不能退回旧 thread 续聊");

assertDeepEqual(
  normalizeRoleSessionReadError(new Error("M3_BINDING_UNAVAILABLE")),
  {
    code: "M3_BINDING_UNAVAILABLE",
    user_message: "角色会话绑定尚未就绪；历史内容仅供阅读，当前不能续聊。",
  },
  "production runtime unavailable 必须给明确闭锁文案",
);

console.log("role-session-read-model: DTO deny_unknown、nonce/epoch、opaque continuation 与闭锁状态离线断言全过");

// M3C06 renderer-side RoleSession read-model boundary.
//
// The renderer may route a request with a canonical project locator and opaque
// selectors only. It never owns actor/role/scope/object/channel/permission,
// provider handle, owner fingerprint, profile, or legacy thread truth.

export const M3_BINDING_UNAVAILABLE = "M3_BINDING_UNAVAILABLE" as const;

export type RoleSessionPermissionState = "CURRENT" | "REVALIDATION_REQUIRED";
export type RoleSessionContextState = "AVAILABLE" | "MISSING" | "NEEDS_REPROJECTION" | "SESSION_FAIL_CLOSED";
export type RoleSessionContinuationState = "AVAILABLE" | "DISABLED";

export type RoleSessionDisplayLabels = Readonly<{
  role_label: string;
  project_label: string;
  object_label: string;
  channel_label: string;
  permission_label: string;
}>;

export type RoleSessionSourceLink = Readonly<{
  source_ref: string | null;
  label: string;
}>;

export type RoleSessionContext = Readonly<{
  state: RoleSessionContextState;
  retrieval_status: string | null;
  context_sources: readonly string[];
  knowledge_refs: readonly string[];
  gaps: readonly string[];
  source_links: readonly RoleSessionSourceLink[];
  request_more_material_available: boolean;
}>;

export type RoleSessionContinuation = Readonly<{
  state: RoleSessionContinuationState;
  selector: string | null;
  reason: string | null;
}>;

export type RoleSessionDirectoryEntry = Readonly<{
  selection: string;
  role_session_id: string;
  session_revision: number;
  labels: RoleSessionDisplayLabels;
  session_state: string;
  permission_state: RoleSessionPermissionState;
  resolution_reason: string | null;
}>;

export type RoleSessionDirectory = Readonly<{
  request_nonce: string;
  projection_revision: string;
  entries: readonly RoleSessionDirectoryEntry[];
  next_cursor: string | null;
}>;

export type RoleSessionDetail = Readonly<{
  request_nonce: string;
  selection: string;
  role_session_id: string;
  session_revision: number;
  projection_revision: string;
  labels: RoleSessionDisplayLabels;
  session_state: string;
  permission_state: RoleSessionPermissionState;
  resolution_reason: string | null;
  context: RoleSessionContext;
  continuation: RoleSessionContinuation;
}>;

export type RoleSessionDirectoryRequest = Readonly<{
  project_locator: string;
  cursor?: string | null;
  limit?: number | null;
  request_nonce: string;
}>;

export type RoleSessionDetailRequest = Readonly<{
  project_locator: string;
  selection: string;
  request_nonce: string;
}>;

export type RoleSessionContinuationStartRequest = Readonly<{
  project_locator: string;
  continuation_selector: string;
  request_nonce: string;
  user_text: string;
}>;

export type RoleSessionReadError = Readonly<{
  code: string;
  user_message: string;
}>;

export type RoleSessionDirectorySelectionStatus = "empty" | "automatic" | "explicit" | "selection_required";

export type RoleSessionDirectorySelectionResolution = Readonly<{
  status: RoleSessionDirectorySelectionStatus;
  selection: string | null;
  rejected_selection: boolean;
}>;

const FORBIDDEN_RENDERER_TRUTH_FIELDS = new Set([
  "actor",
  "actor_id",
  "role",
  "role_ref",
  "scope",
  "scope_ref",
  "object",
  "current_object_ref",
  "channel",
  "execution_channel",
  "permission",
  "permission_snapshot_ref",
  "owner",
  "owner_fingerprint",
  "provider_handle",
  "provider_handle_ref",
  "profile",
  "profile_id",
  "surface",
  "thread",
  "thread_id",
  "conversation_id",
]);

let nonceSequence = 0;

export function createRoleSessionRequestNonce(prefix = "m3rs-read"): string {
  nonceSequence += 1;
  return `${prefix}-${Date.now().toString(36)}-${nonceSequence.toString(36)}`;
}

export function createRoleSessionReadEpoch() {
  let current = 0;
  return Object.freeze({
    next(): number {
      current += 1;
      return current;
    },
    isCurrent(epoch: number): boolean {
      return current === epoch;
    },
  });
}

export function createRoleSessionDirectoryRequest(input: RoleSessionDirectoryRequest): RoleSessionDirectoryRequest {
  assertRequestShape(input, ["project_locator", "cursor", "limit", "request_nonce"]);
  const project_locator = requiredString(input.project_locator, "project_locator");
  const request_nonce = requiredString(input.request_nonce, "request_nonce");
  const cursor = optionalString(input.cursor, "cursor");
  const limit = input.limit === undefined || input.limit === null ? undefined : requiredLimit(input.limit);
  return Object.freeze({ project_locator, cursor, limit, request_nonce });
}

export function createRoleSessionDetailRequest(input: RoleSessionDetailRequest): RoleSessionDetailRequest {
  assertRequestShape(input, ["project_locator", "selection", "request_nonce"]);
  return Object.freeze({
    project_locator: requiredString(input.project_locator, "project_locator"),
    selection: requiredString(input.selection, "selection"),
    request_nonce: requiredString(input.request_nonce, "request_nonce"),
  });
}

export function createRoleSessionContinuationStartRequest(
  input: RoleSessionContinuationStartRequest,
): RoleSessionContinuationStartRequest {
  assertRequestShape(input, ["project_locator", "continuation_selector", "request_nonce", "user_text"]);
  return Object.freeze({
    project_locator: requiredString(input.project_locator, "project_locator"),
    continuation_selector: requiredString(input.continuation_selector, "continuation_selector"),
    request_nonce: requiredString(input.request_nonce, "request_nonce"),
    user_text: requiredString(input.user_text, "user_text"),
  });
}

export function parseRoleSessionDirectory(value: unknown): RoleSessionDirectory {
  const raw = exactObject(value, ["request_nonce", "projection_revision", "entries", "next_cursor"], "directory");
  return Object.freeze({
    request_nonce: requiredString(raw.request_nonce, "directory.request_nonce"),
    projection_revision: requiredString(raw.projection_revision, "directory.projection_revision"),
    entries: readonlyArray(raw.entries, "directory.entries").map((entry, index) => parseDirectoryEntry(entry, index)),
    next_cursor: optionalString(raw.next_cursor, "directory.next_cursor"),
  });
}

export function parseRoleSessionDetail(value: unknown): RoleSessionDetail {
  const raw = exactObject(
    value,
    [
      "request_nonce",
      "selection",
      "role_session_id",
      "session_revision",
      "projection_revision",
      "labels",
      "session_state",
      "permission_state",
      "resolution_reason",
      "context",
      "continuation",
    ],
    "detail",
  );
  return Object.freeze({
    request_nonce: requiredString(raw.request_nonce, "detail.request_nonce"),
    selection: requiredString(raw.selection, "detail.selection"),
    role_session_id: requiredString(raw.role_session_id, "detail.role_session_id"),
    session_revision: requiredNonNegativeInteger(raw.session_revision, "detail.session_revision"),
    projection_revision: requiredString(raw.projection_revision, "detail.projection_revision"),
    labels: parseLabels(raw.labels, "detail.labels"),
    session_state: requiredString(raw.session_state, "detail.session_state"),
    permission_state: parsePermissionState(raw.permission_state, "detail.permission_state"),
    resolution_reason: optionalString(raw.resolution_reason, "detail.resolution_reason"),
    context: parseContext(raw.context),
    continuation: parseContinuation(raw.continuation),
  });
}

export function roleSessionDetailMatchesRequest(
  detail: RoleSessionDetail,
  request: Pick<RoleSessionDetailRequest, "selection" | "request_nonce">,
): boolean {
  return detail.selection === request.selection && detail.request_nonce === request.request_nonce;
}

export function roleSessionDirectoryMatchesRequest(
  directory: RoleSessionDirectory,
  request: Pick<RoleSessionDirectoryRequest, "request_nonce">,
): boolean {
  return directory.request_nonce === request.request_nonce;
}

// Directory order is history presentation only.  A renderer may auto-select
// exactly one complete server directory entry, or an explicit opaque entry
// selection from that same directory. It must never turn the first list item into an
// identity when more data exists.
export function resolveRoleSessionDirectorySelection(
  directory: RoleSessionDirectory | null | undefined,
  requestedSelection: string | null | undefined = null,
): RoleSessionDirectorySelectionResolution {
  if (!directory || directory.entries.length === 0) {
    return Object.freeze({ status: "empty", selection: null, rejected_selection: false });
  }
  const requested = typeof requestedSelection === "string" && requestedSelection.trim() ? requestedSelection : null;
  if (requested) {
    if (directory.entries.some((entry) => entry.selection === requested)) {
      return Object.freeze({ status: "explicit", selection: requested, rejected_selection: false });
    }
    return Object.freeze({ status: "selection_required", selection: null, rejected_selection: true });
  }
  if (directory.entries.length === 1 && directory.next_cursor === null) {
    const [onlyEntry] = directory.entries;
    if (onlyEntry) {
      return Object.freeze({ status: "automatic", selection: onlyEntry.selection, rejected_selection: false });
    }
  }
  return Object.freeze({ status: "selection_required", selection: null, rejected_selection: false });
}

export function roleSessionDirectoryHasSelection(
  directory: RoleSessionDirectory | null | undefined,
  selection: string | null | undefined,
): boolean {
  return Boolean(
    selection
      && directory?.entries.some((entry) => entry.selection === selection),
  );
}

export function mergeRoleSessionDirectoryPage(
  directory: RoleSessionDirectory,
  page: RoleSessionDirectory,
): RoleSessionDirectory {
  const seen = new Set<string>();
  const entries: RoleSessionDirectoryEntry[] = [];
  for (const entry of [...directory.entries, ...page.entries]) {
    if (seen.has(entry.selection)) continue;
    seen.add(entry.selection);
    entries.push(entry);
  }
  return Object.freeze({
    request_nonce: page.request_nonce,
    projection_revision: page.projection_revision,
    entries: Object.freeze(entries),
    next_cursor: page.next_cursor,
  });
}

// A directory page revision is deliberately scoped to that page, while a
// detail revision is scoped to one RoleSession snapshot. They are different
// revision domains, so comparing those opaque strings would reject every valid
// second page. The current protocol has no whole-directory snapshot token.
// We therefore only merge pages when every piece of overlapping server
// evidence agrees, and clear the current selection when an observable drift is
// found. A server-wide pagination snapshot remains a future protocol addition.
export function roleSessionDirectoryPageHasCompatibleProjection(
  directory: RoleSessionDirectory,
  page: RoleSessionDirectory,
  currentDetail: RoleSessionDetail | null | undefined,
): boolean {
  if (!directory.projection_revision.trim() || !page.projection_revision.trim()) return false;
  const bySelection = new Map(directory.entries.map((entry) => [entry.selection, entry]));
  const byRoleSessionId = new Map(directory.entries.map((entry) => [entry.role_session_id, entry]));

  for (const entry of page.entries) {
    const sameSelection = bySelection.get(entry.selection);
    if (sameSelection && !sameDirectoryEntry(sameSelection, entry)) return false;
    const sameRoleSession = byRoleSessionId.get(entry.role_session_id);
    if (sameRoleSession && !sameDirectoryEntry(sameRoleSession, entry)) return false;
  }

  return !currentDetail || roleSessionDetailMatchesDirectoryEntry(currentDetail, directory);
}

export function roleSessionDetailMatchesCurrentSelection(
  detail: RoleSessionDetail | null | undefined,
  request: Pick<RoleSessionDetailRequest, "selection" | "request_nonce">,
  directory: RoleSessionDirectory | null | undefined,
  selectedSelection: string | null | undefined,
): boolean {
  return Boolean(
    detail
      && selectedSelection
      && detail.selection === selectedSelection
      && roleSessionDetailMatchesRequest(detail, request)
      && roleSessionDirectoryHasSelection(directory, selectedSelection)
      && roleSessionDetailMatchesDirectoryEntry(detail, directory),
  );
}

export function roleSessionDetailMatchesDirectoryEntry(
  detail: RoleSessionDetail | null | undefined,
  directory: RoleSessionDirectory | null | undefined,
): boolean {
  if (!detail || !directory) return false;
  const entry = directory.entries.find((candidate) => candidate.selection === detail.selection);
  return Boolean(
    entry
      && entry.role_session_id === detail.role_session_id
      && entry.session_revision === detail.session_revision
      && sameLabels(entry.labels, detail.labels)
      && entry.session_state === detail.session_state
      && entry.permission_state === detail.permission_state
      && entry.resolution_reason === detail.resolution_reason,
  );
}

export function usableRoleSessionContinuationSelector(detail: RoleSessionDetail | null | undefined): string | null {
  if (detail?.continuation.state !== "AVAILABLE") return null;
  const selector = detail.continuation.selector?.trim() ?? "";
  return selector || null;
}

export function usableCurrentRoleSessionContinuationSelector(
  detail: RoleSessionDetail | null | undefined,
  selectedSelection: string | null | undefined,
  directory: RoleSessionDirectory | null | undefined,
): string | null {
  if (!selectedSelection || detail?.selection !== selectedSelection || !roleSessionDetailMatchesDirectoryEntry(detail, directory)) {
    return null;
  }
  return usableRoleSessionContinuationSelector(detail);
}

export function normalizeRoleSessionReadError(error: unknown): RoleSessionReadError {
  const code = error instanceof Error ? error.message : typeof error === "string" ? error : "M3_READ_MODEL_UNAVAILABLE";
  if (code.includes(M3_BINDING_UNAVAILABLE)) {
    return Object.freeze({
      code: M3_BINDING_UNAVAILABLE,
      user_message: "角色会话绑定尚未就绪；历史内容仅供阅读，当前不能续聊。",
    });
  }
  if (code.includes("PERMISSION_REVALIDATION_REQUIRED")) {
    return Object.freeze({
      code: "PERMISSION_REVALIDATION_REQUIRED",
      user_message: "会话权限发生变化，等待服务端重新验证。",
    });
  }
  return Object.freeze({
    code: scrubErrorCode(code),
    user_message: "角色会话读取暂时不可用；没有使用本地缓存续聊。",
  });
}

function parseDirectoryEntry(value: unknown, index: number): RoleSessionDirectoryEntry {
  const raw = exactObject(
    value,
    ["selection", "role_session_id", "session_revision", "labels", "session_state", "permission_state", "resolution_reason"],
    `directory.entries[${index}]`,
  );
  return Object.freeze({
    selection: requiredString(raw.selection, `directory.entries[${index}].selection`),
    role_session_id: requiredString(raw.role_session_id, `directory.entries[${index}].role_session_id`),
    session_revision: requiredNonNegativeInteger(raw.session_revision, `directory.entries[${index}].session_revision`),
    labels: parseLabels(raw.labels, `directory.entries[${index}].labels`),
    session_state: requiredString(raw.session_state, `directory.entries[${index}].session_state`),
    permission_state: parsePermissionState(raw.permission_state, `directory.entries[${index}].permission_state`),
    resolution_reason: optionalString(raw.resolution_reason, `directory.entries[${index}].resolution_reason`),
  });
}

function sameDirectoryEntry(left: RoleSessionDirectoryEntry, right: RoleSessionDirectoryEntry): boolean {
  return left.role_session_id === right.role_session_id
    && left.session_revision === right.session_revision
    && sameLabels(left.labels, right.labels)
    && left.session_state === right.session_state
    && left.permission_state === right.permission_state
    && left.resolution_reason === right.resolution_reason;
}

function sameLabels(left: RoleSessionDisplayLabels, right: RoleSessionDisplayLabels): boolean {
  return left.role_label === right.role_label
    && left.project_label === right.project_label
    && left.object_label === right.object_label
    && left.channel_label === right.channel_label
    && left.permission_label === right.permission_label;
}

function parseLabels(value: unknown, field: string): RoleSessionDisplayLabels {
  const raw = exactObject(
    value,
    ["role_label", "project_label", "object_label", "channel_label", "permission_label"],
    field,
  );
  return Object.freeze({
    role_label: requiredString(raw.role_label, `${field}.role_label`),
    project_label: requiredString(raw.project_label, `${field}.project_label`),
    object_label: requiredString(raw.object_label, `${field}.object_label`),
    channel_label: requiredString(raw.channel_label, `${field}.channel_label`),
    permission_label: requiredString(raw.permission_label, `${field}.permission_label`),
  });
}

function parseContext(value: unknown): RoleSessionContext {
  const raw = exactObject(
    value,
    [
      "state",
      "retrieval_status",
      "context_sources",
      "knowledge_refs",
      "gaps",
      "source_links",
      "request_more_material_available",
    ],
    "detail.context",
  );
  return Object.freeze({
    state: parseContextState(raw.state, "detail.context.state"),
    retrieval_status: optionalString(raw.retrieval_status, "detail.context.retrieval_status"),
    context_sources: readonlyStringArray(raw.context_sources, "detail.context.context_sources"),
    knowledge_refs: readonlyStringArray(raw.knowledge_refs, "detail.context.knowledge_refs"),
    gaps: readonlyStringArray(raw.gaps, "detail.context.gaps"),
    source_links: readonlyArray(raw.source_links, "detail.context.source_links").map((link, index) => {
      const source = exactObject(link, ["source_ref", "label"], `detail.context.source_links[${index}]`);
      return Object.freeze({
        source_ref: optionalString(source.source_ref, `detail.context.source_links[${index}].source_ref`),
        label: requiredString(source.label, `detail.context.source_links[${index}].label`),
      });
    }),
    request_more_material_available: requiredBoolean(
      raw.request_more_material_available,
      "detail.context.request_more_material_available",
    ),
  });
}

function parseContinuation(value: unknown): RoleSessionContinuation {
  const raw = exactObject(value, ["state", "selector", "reason"], "detail.continuation");
  const state = parseContinuationState(raw.state, "detail.continuation.state");
  const selector = optionalString(raw.selector, "detail.continuation.selector");
  if (state === "AVAILABLE" && !selector) throw new Error("m3_read_model_invalid_available_continuation");
  if (state === "DISABLED" && selector) throw new Error("m3_read_model_invalid_disabled_continuation");
  return Object.freeze({
    state,
    selector,
    reason: optionalString(raw.reason, "detail.continuation.reason"),
  });
}

function parsePermissionState(value: unknown, field: string): RoleSessionPermissionState {
  if (value === "CURRENT" || value === "REVALIDATION_REQUIRED") return value;
  throw new Error(`m3_read_model_invalid_${field}`);
}

function parseContextState(value: unknown, field: string): RoleSessionContextState {
  if (value === "AVAILABLE" || value === "MISSING" || value === "NEEDS_REPROJECTION" || value === "SESSION_FAIL_CLOSED") {
    return value;
  }
  throw new Error(`m3_read_model_invalid_${field}`);
}

function parseContinuationState(value: unknown, field: string): RoleSessionContinuationState {
  if (value === "AVAILABLE" || value === "DISABLED") return value;
  throw new Error(`m3_read_model_invalid_${field}`);
}

function assertRequestShape(value: unknown, allowedKeys: readonly string[]) {
  const raw = exactObject(value, allowedKeys, "request");
  for (const key of Object.keys(raw)) {
    if (FORBIDDEN_RENDERER_TRUTH_FIELDS.has(key)) throw new Error(`m3_read_model_forbidden_request_field:${key}`);
  }
}

function exactObject(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`m3_read_model_invalid_${field}`);
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m3_read_model_unknown_${field}_field:${key}`);
  }
  return raw;
}

function readonlyArray(value: unknown, field: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`m3_read_model_invalid_${field}`);
  return Object.freeze([...value]);
}

function readonlyStringArray(value: unknown, field: string): readonly string[] {
  return Object.freeze(readonlyArray(value, field).map((entry, index) => requiredString(entry, `${field}[${index}]`)));
}

function requiredString(value: unknown, field: string): string {
  if (typeof value !== "string" || !value.trim()) throw new Error(`m3_read_model_invalid_${field}`);
  return value;
}

function optionalString(value: unknown, field: string): string | null {
  if (value === undefined || value === null) return null;
  return requiredString(value, field);
}

function requiredNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error(`m3_read_model_invalid_${field}`);
  }
  return value;
}

function requiredLimit(value: unknown): number {
  const limit = requiredNonNegativeInteger(value, "request.limit");
  if (limit < 1 || limit > 100) throw new Error("m3_read_model_invalid_request.limit");
  return limit;
}

function requiredBoolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") throw new Error(`m3_read_model_invalid_${field}`);
  return value;
}

function scrubErrorCode(value: string): string {
  return value.includes("m3_role_session")
    ? "M3_ROLE_SESSION_READ_REJECTED"
    : "M3_READ_MODEL_UNAVAILABLE";
}

import Foundation
import AppKit
import ScreenCaptureKit
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

struct CaptureArgs {
    var mode: String
    var title: String?
    var windowId: UInt32?
    var output: String?
}

func parseArgs(_ args: [String]) throws -> CaptureArgs {
    var result = CaptureArgs(mode: "list", title: nil, windowId: nil, output: nil)
    var index = 1
    while index < args.count {
        let arg = args[index]
        switch arg {
        case "--list":
            result.mode = "list"
        case "--title":
            guard index + 1 < args.count else { throw NSError(domain: "args", code: 1) }
            result.title = args[index + 1]
            index += 1
        case "--capture-title":
            guard index + 1 < args.count else { throw NSError(domain: "args", code: 2) }
            result.mode = "capture-title"
            result.title = args[index + 1]
            index += 1
        case "--capture-window-id":
            guard index + 1 < args.count, let parsed = UInt32(args[index + 1]) else {
                throw NSError(domain: "args", code: 3)
            }
            result.mode = "capture-window-id"
            result.windowId = parsed
            index += 1
        case "--output":
            guard index + 1 < args.count else { throw NSError(domain: "args", code: 4) }
            result.output = args[index + 1]
            index += 1
        default:
            throw NSError(domain: "args", code: 5, userInfo: [NSLocalizedDescriptionKey: "Unknown argument: \(arg)"])
        }
        index += 1
    }
    return result
}

func loadWindows() async throws -> [SCWindow] {
    let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
    return content.windows.sorted { left, right in
        String(describing: left.title) < String(describing: right.title)
    }
}

func matches(_ window: SCWindow, titleNeedle: String?) -> Bool {
    guard let titleNeedle, !titleNeedle.isEmpty else { return true }
    let title = window.title ?? ""
    let appName = window.owningApplication?.applicationName ?? ""
    let bundleId = window.owningApplication?.bundleIdentifier ?? ""
    return title.localizedCaseInsensitiveContains(titleNeedle)
        || appName.localizedCaseInsensitiveContains(titleNeedle)
        || bundleId.localizedCaseInsensitiveContains(titleNeedle)
}

func printWindow(_ window: SCWindow) {
    let app = window.owningApplication
    let title = window.title ?? ""
    let appName = app?.applicationName ?? ""
    let bundleId = app?.bundleIdentifier ?? ""
    let pid = app?.processID ?? -1
    let frame = window.frame
    print("window_id=\(window.windowID) pid=\(pid) app=\"\(appName)\" bundle=\"\(bundleId)\" title=\"\(title)\" frame={x:\(Int(frame.origin.x)),y:\(Int(frame.origin.y)),w:\(Int(frame.width)),h:\(Int(frame.height))}")
}

func writePng(_ image: CGImage, output: String) throws {
    let url = URL(fileURLWithPath: output)
    guard let destination = CGImageDestinationCreateWithURL(
        url as CFURL,
        UTType.png.identifier as CFString,
        1,
        nil
    ) else {
        throw NSError(domain: "png", code: 1, userInfo: [NSLocalizedDescriptionKey: "Cannot create PNG destination"])
    }
    CGImageDestinationAddImage(destination, image, nil)
    guard CGImageDestinationFinalize(destination) else {
        throw NSError(domain: "png", code: 2, userInfo: [NSLocalizedDescriptionKey: "Cannot finalize PNG"])
    }
}

func capture(_ window: SCWindow, output: String) async throws {
    let config = SCStreamConfiguration()
    config.width = max(1, Int(window.frame.width))
    config.height = max(1, Int(window.frame.height))
    config.showsCursor = false
    config.capturesAudio = false

    let filter = SCContentFilter(desktopIndependentWindow: window)
    let image = try await SCScreenshotManager.captureImage(
        contentFilter: filter,
        configuration: config
    )
    try writePng(image, output: output)
    print("captured window_id=\(window.windowID) output=\"\(output)\" size=\(image.width)x\(image.height)")
}

@main
struct StageKScreenCaptureKitWindowCapture {
    static func main() async {
        do {
            _ = NSApplication.shared
            let args = try parseArgs(CommandLine.arguments)
            let windows = try await loadWindows()

            if args.mode == "list" {
                windows.filter { matches($0, titleNeedle: args.title) }.forEach(printWindow)
                return
            }

            guard let output = args.output else {
                throw NSError(domain: "args", code: 6, userInfo: [NSLocalizedDescriptionKey: "--output is required for capture"])
            }

            let target: SCWindow?
            if args.mode == "capture-window-id", let windowId = args.windowId {
                target = windows.first { $0.windowID == windowId }
            } else {
                target = windows.first { matches($0, titleNeedle: args.title) }
            }

            guard let target else {
                throw NSError(domain: "capture", code: 1, userInfo: [NSLocalizedDescriptionKey: "Target window not found"])
            }

            printWindow(target)
            try await capture(target, output: output)
        } catch {
            fputs("stage-k-screencapturekit-window-capture failed: \(error.localizedDescription)\n", stderr)
            Foundation.exit(1)
        }
    }
}

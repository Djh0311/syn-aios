import Foundation
import CoreGraphics

let args = CommandLine.arguments
guard args.count == 3, let x = Double(args[1]), let y = Double(args[2]) else {
    fputs("usage: stage-k-cgevent-click <x> <y>\n", stderr)
    Foundation.exit(1)
}

let point = CGPoint(x: x, y: y)
guard
    let mouseDown = CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown, mouseCursorPosition: point, mouseButton: .left),
    let mouseUp = CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp, mouseCursorPosition: point, mouseButton: .left)
else {
    fputs("failed to create CGEvent\n", stderr)
    Foundation.exit(1)
}

mouseDown.post(tap: .cghidEventTap)
usleep(80_000)
mouseUp.post(tap: .cghidEventTap)

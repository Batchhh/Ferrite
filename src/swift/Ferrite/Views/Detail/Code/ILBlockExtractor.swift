import SwiftUI

// MARK: - IL Block Extraction

extension CodePreviewView {
    /// Extract a `.property` block and its accessor `.method` blocks from the full type IL.
    func extractPropertyIL(_ il: String, property: PropertyInfo, type_: TypeInfo) -> String {
        var blocks: [String] = []

        // Extract the .property block
        if let block = extractILBlock(il, directive: ".property", name: property.name) {
            blocks.append(block)
        }

        // Extract getter method block
        if let getterToken = property.getterToken,
           let getter = type_.members.first(where: { $0.token == getterToken }) {
            if let block = extractILBlock(il, directive: ".method", name: getter.name) {
                blocks.append(block)
            }
        }

        // Extract setter method block
        if let setterToken = property.setterToken,
           let setter = type_.members.first(where: { $0.token == setterToken }) {
            if let block = extractILBlock(il, directive: ".method", name: setter.name) {
                blocks.append(block)
            }
        }

        return blocks.isEmpty ? il : blocks.joined(separator: "\n")
    }

    /// Extract an `.event` block and its accessor `.method` blocks from the full type IL.
    func extractEventIL(_ il: String, event: EventInfo, type_: TypeInfo) -> String {
        var blocks: [String] = []

        // Extract the .event block
        if let block = extractILBlock(il, directive: ".event", name: event.name) {
            blocks.append(block)
        }

        // Extract accessor method blocks
        let accessorTokens = [event.addToken, event.removeToken, event.raiseToken].compactMap { $0 }
        for token in accessorTokens {
            if let method = type_.members.first(where: { $0.token == token }),
               let block = extractILBlock(il, directive: ".method", name: method.name) {
                blocks.append(block)
            }
        }

        return blocks.isEmpty ? il : blocks.joined(separator: "\n")
    }

    /// Extract a single IL block (e.g. `.property ... Name ...{ ... }`) from the full IL text.
    func extractILBlock(_ il: String, directive: String, name: String) -> String? {
        let lines = il.components(separatedBy: "\n")
        var result: [String] = []
        var depth = 0
        var capturing = false

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if !capturing {
                if trimmed.hasPrefix(directive) && trimmed.contains(" \(name)") {
                    capturing = true
                    result.append(line)
                    if trimmed.contains("{") { depth += 1 }
                }
            } else {
                result.append(line)
                for ch in trimmed {
                    if ch == "{" { depth += 1 }
                    else if ch == "}" { depth -= 1 }
                }
                if depth <= 0 { break }
            }
        }

        return capturing ? result.joined(separator: "\n") : nil
    }
}

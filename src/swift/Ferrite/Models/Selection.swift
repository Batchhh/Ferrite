import Foundation

/// Identifies the currently selected item in the assembly browser.
enum Selection: Hashable, Codable {
    case assembly(id: String)
    case namespace(assemblyId: String, name: String)
    case type(assemblyId: String, token: UInt32)
    case member(assemblyId: String, typeToken: UInt32, memberToken: UInt32)
}

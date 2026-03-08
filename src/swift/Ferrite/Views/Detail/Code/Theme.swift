import SwiftUI

// Requires macOS 26 (Tahoe) for Liquid Glass APIs.

// MARK: - Theme Colors

extension Color {
    // Panel backgrounds
    static let sidebarBackground = Color(red: 0.13, green: 0.13, blue: 0.14)
    static let contentBackground = Color(red: 0.10, green: 0.10, blue: 0.11)

    // Code syntax — Cursor Dark theme
    static let codeBackground    = Color(red: 0.07, green: 0.09, blue: 0.13)
    static let codeKeyword       = Color(red: 0.34, green: 0.61, blue: 0.84)  // #569CD6
    static let codeControlFlow   = Color(red: 0.77, green: 0.53, blue: 0.75)  // #C586C0
    static let codeType          = Color(red: 0.31, green: 0.79, blue: 0.69)  // #4EC9B0
    static let codeInterface     = Color(red: 0.72, green: 0.84, blue: 0.64)  // #B8D7A3
    static let codeEnumName      = Color(red: 0.72, green: 0.84, blue: 0.64)  // #B8D7A3
    static let codeMethod        = Color(red: 0.86, green: 0.86, blue: 0.67)  // #DCDCAA
    static let codeField         = Color(red: 0.61, green: 0.86, blue: 1.00)  // #9CDCFE
    static let codeComment       = Color(red: 0.42, green: 0.60, blue: 0.33)  // #6A9955
    static let codeXmlDoc        = Color(red: 0.38, green: 0.55, blue: 0.31)  // #608B4E
    static let codeString        = Color(red: 0.81, green: 0.57, blue: 0.47)  // #CE9178
    static let codeNumber        = Color(red: 0.71, green: 0.81, blue: 0.66)  // #B5CEA8
    static let codePreprocessor  = Color(white: 0.50)                          // #808080
    static let codePlain         = Color(red: 0.83, green: 0.83, blue: 0.83)  // #D4D4D4
    static let codeCollapsed     = Color(white: 0.45)

    // Search highlights
    static let codeSearchMatch        = Color(red: 0.90, green: 0.65, blue: 0.15, opacity: 0.3)
    static let codeSearchCurrentMatch = Color(red: 0.90, green: 0.65, blue: 0.15, opacity: 0.7)
}

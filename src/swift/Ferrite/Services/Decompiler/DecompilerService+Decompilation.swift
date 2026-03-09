import SwiftUI

// MARK: - Lazy loading & decompilation

extension DecompilerService {
    /// Fetch type summaries for a namespace (cached after first call).
    func getNamespaceTypes(assemblyId: String, namespace: String) -> [TypeSummary] {
        let key = "\(assemblyId):\(namespace)"
        if let cached = namespaceTypesCache[key] { return cached }
        do {
            let types = try session.getNamespaceTypes(assemblyId: assemblyId, namespace: namespace)
            namespaceTypesCache[key] = types
            return types
        } catch {
            self.error = "Failed to fetch namespace types: \(error.localizedDescription)"
            return []
        }
    }

    /// Fetch full type details by token (cached after first call).
    func getTypeDetails(assemblyId: String, token: UInt32) -> TypeInfo? {
        let key = "\(assemblyId):\(token)"
        if let cached = typeDetailsCache[key] { return cached }
        do {
            let info = try session.getTypeDetails(assemblyId: assemblyId, typeToken: token)
            typeDetailsCache[key] = info
            return info
        } catch {
            self.error = "Failed to fetch type details: \(error.localizedDescription)"
            return nil
        }
    }

    // MARK: - Lookup helpers

    func findAssembly(id: String) -> AssemblySummary? {
        loadedAssemblies.first { $0.id == id }
    }

    func findType(assemblyId: String, token: UInt32) -> TypeInfo? {
        getTypeDetails(assemblyId: assemblyId, token: token)
    }

    func findMember(assemblyId: String, typeToken: UInt32, memberToken: UInt32) -> MemberInfo? {
        guard let type_ = findType(assemblyId: assemblyId, token: typeToken) else { return nil }
        return type_.members.first { $0.token == memberToken }
    }

    func findProperty(assemblyId: String, typeToken: UInt32, propertyToken: UInt32) -> PropertyInfo? {
        guard let type_ = findType(assemblyId: assemblyId, token: typeToken) else { return nil }
        return type_.properties.first { $0.token == propertyToken }
    }

    func findEvent(assemblyId: String, typeToken: UInt32, eventToken: UInt32) -> EventInfo? {
        guard let type_ = findType(assemblyId: assemblyId, token: typeToken) else { return nil }
        return type_.events.first { $0.token == eventToken }
    }

    func findTypeByShortName(_ name: String, preferredAssemblyId: String? = nil) -> (assemblyId: String, token: UInt32)? {
        guard let entries = typeNameCache[name], !entries.isEmpty else { return nil }
        if let preferred = preferredAssemblyId,
           let match = entries.first(where: { $0.assemblyId == preferred }) {
            return match
        }
        return entries.first
    }

    /// Decompile a type to C# source, returning `nil` on failure.
    func decompileType(assemblyId: String, token: UInt32) -> String? {
        do {
            return try session.decompileType(assemblyId: assemblyId, typeToken: token)
        } catch {
            self.error = "Decompilation failed: \(error.localizedDescription)"
            return nil
        }
    }

    /// Disassemble a type to IL text, returning `nil` on failure.
    func disassembleTypeIL(assemblyId: String, token: UInt32) -> String? {
        do {
            return try session.disassembleTypeIl(assemblyId: assemblyId, typeToken: token)
        } catch {
            self.error = "IL disassembly failed: \(error.localizedDescription)"
            return nil
        }
    }
}

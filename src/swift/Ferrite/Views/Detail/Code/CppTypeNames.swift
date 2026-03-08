import AppKit

// MARK: - Header Export (Fields-Only Recursive C++ Header)

/// Well-known BCL namespaces whose types should not be recursed into.
private let bclNamespacePrefixes: [String] = [
    "System.", "System", "Microsoft.", "Microsoft",
]

func isBCLType(_ fullName: String) -> Bool {
    bclNamespacePrefixes.contains { fullName.hasPrefix($0) }
}

/// Well-known BCL short names that should be skipped from C++ inheritance.
let bclSkipNames: Set<String> = [
    // Primitive / root
    "void*", "Object", "ValueType", "Enum",
    // Delegates
    "Delegate", "MulticastDelegate",
    // Exceptions
    "Exception", "SystemException", "ApplicationException",
    "ArgumentException", "ArgumentNullException", "ArgumentOutOfRangeException",
    "InvalidOperationException", "NotSupportedException", "NotImplementedException",
    "NullReferenceException", "IndexOutOfRangeException", "InvalidCastException",
    "ArithmeticException", "OverflowException", "DivideByZeroException",
    "FormatException", "ObjectDisposedException", "TimeoutException",
    "IOException", "FileNotFoundException", "DirectoryNotFoundException",
    "UnauthorizedAccessException", "SecurityException",
    "AggregateException", "OperationCanceledException", "TaskCanceledException",
    "StackOverflowException", "OutOfMemoryException", "AccessViolationException",
    "TypeLoadException", "TypeInitializationException", "MissingMethodException",
    "MissingFieldException", "MissingMemberException", "EntryPointNotFoundException",
    "BadImageFormatException", "KeyNotFoundException",
    "SerializationException", "DecoderFallbackException", "EncoderFallbackException",
    "ExternalException", "COMException", "SEHException", "Win32Exception",
    "HttpRequestException", "WebException", "SocketException",
    // Attributes
    "Attribute", "FlagsAttribute", "ObsoleteAttribute", "SerializableAttribute",
    "ConditionalAttribute", "DebuggerDisplayAttribute", "DebuggerHiddenAttribute",
    "DebuggerStepThroughAttribute", "DebuggerBrowsableAttribute",
    "CompilerGeneratedAttribute", "CallerMemberNameAttribute",
    "CallerFilePathAttribute", "CallerLineNumberAttribute",
    "DefaultValueAttribute", "EditorBrowsableAttribute",
    "BrowsableAttribute", "CategoryAttribute", "DescriptionAttribute",
    "DisplayNameAttribute", "DataContractAttribute", "DataMemberAttribute",
    "XmlRootAttribute", "XmlElementAttribute", "XmlAttributeAttribute",
    "JsonPropertyAttribute", "JsonIgnoreAttribute",
    // Reflection / runtime
    "MarshalByRefObject", "ContextBoundObject", "CriticalFinalizerObject",
    "MemberInfo", "MethodBase", "MethodInfo", "ConstructorInfo",
    "FieldInfo", "PropertyInfo", "EventInfo", "ParameterInfo",
    "Assembly", "Module", "Type",
    // Events / args
    "EventArgs", "CancelEventArgs", "PropertyChangedEventArgs",
    "NotifyCollectionChangedEventArgs", "RoutedEventArgs",
    // Streams / IO
    "Stream", "MemoryStream", "FileStream", "BufferedStream",
    "TextReader", "TextWriter", "StringReader", "StringWriter",
    "StreamReader", "StreamWriter", "BinaryReader", "BinaryWriter",
    // Collections
    "Array", "ArrayList", "Hashtable", "CollectionBase",
    "ReadOnlyCollectionBase", "DictionaryBase",
    // Threading
    "WaitHandle", "Mutex", "Semaphore", "EventWaitHandle",
    "ManualResetEvent", "AutoResetEvent", "Timer",
    "SynchronizationContext", "TaskScheduler",
    "CancellationTokenSource",
    // Text / encoding
    "Encoding", "Encoder", "Decoder",
    "StringBuilder",
    // Interfaces
    "IDisposable", "ICloneable", "ISerializable",
    "IComparable", "IEquatable", "IFormattable", "IConvertible",
    "IEnumerable", "IEnumerator", "ICollection", "IList", "IDictionary",
    "IReadOnlyCollection", "IReadOnlyList", "IReadOnlyDictionary",
    "ISet", "IComparer", "IEqualityComparer",
    "INotifyPropertyChanged", "INotifyCollectionChanged",
    "INotifyPropertyChanging", "INotifyDataErrorInfo",
    "IDataErrorInfo", "IEditableObject",
    "IAsyncResult", "IAsyncDisposable",
    "IObservable", "IObserver", "IProgress",
    "IStructuralEquatable", "IStructuralComparable",
    // Misc framework
    "Component", "MarshalByValueComponent",
    "DbConnection", "DbCommand", "DbDataReader", "DbParameter",
    "HttpContent", "HttpMessageHandler",
    "Expression", "LambdaExpression",
    "Comparer", "EqualityComparer",
    "CultureInfo", "RegionInfo",
    "Random", "Convert", "BitConverter",
    "Math", "Guid", "Uri", "UriBuilder",
    "WeakReference", "Lazy",
    "Span", "ReadOnlySpan", "Memory", "ReadOnlyMemory",
    "ArraySegment",
]

/// Strip the generic arity suffix from a .NET type name (e.g. `IEquatable`1` → `IEquatable`).
func stripAritySuffix(_ name: String) -> String {
    if let idx = name.firstIndex(of: "`") {
        return String(name[..<idx])
    }
    return name
}

/// Sanitize a .NET name into a valid C++ identifier.
func cppIdentifier(_ name: String) -> String {
    var result = stripAritySuffix(name)
    for ch: Character in ["+", ".", "-", "<", ">", ",", " "] {
        result = result.replacingOccurrences(of: String(ch), with: ch == " " ? "" : "_")
    }
    return result
}

/// Map a C# type name to a C++ type name.
func cppTypeName(_ csType: String) -> String {
    let name = csType.trimmingCharacters(in: .whitespaces)

    if name.hasSuffix("?") {
        return "std::optional<\(cppTypeName(String(name.dropLast(1))))>"
    }
    if name.hasSuffix("[]") {
        return "std::vector<\(cppTypeName(String(name.dropLast(2))))>"
    }

    if let openIdx = findTopLevelGenericOpen(name) {
        let outer = String(name[name.startIndex..<openIdx])
        let argsContent = String(name[name.index(after: openIdx)..<name.index(before: name.endIndex)])
        let args = splitGenericArgs(argsContent).map { cppTypeName($0.trimmingCharacters(in: .whitespaces)) }
        let joined = args.joined(separator: ", ")

        switch outer {
        case "List", "IList", "ICollection", "IEnumerable", "IReadOnlyList", "IReadOnlyCollection":
            return "std::vector<\(joined)>"
        case "Dictionary", "IDictionary", "IReadOnlyDictionary", "SortedDictionary":
            return "std::unordered_map<\(joined)>"
        case "HashSet", "ISet":
            return "std::unordered_set<\(joined)>"
        case "KeyValuePair":
            return "std::pair<\(joined)>"
        case "Nullable":
            return "std::optional<\(joined)>"
        case "Action":
            return "std::function<void(\(joined))>"
        case "Func":
            if args.count > 1 {
                let ret = args.last!
                let params = args.dropLast().joined(separator: ", ")
                return "std::function<\(ret)(\(params))>"
            }
            return "std::function<\(joined)()>"
        case "Tuple", "ValueTuple":
            return "std::tuple<\(joined)>"
        case "Task", "ValueTask":
            return joined
        default:
            return cppIdentifier(outer)
        }
    }

    switch name {
    case "void":    return "void"
    case "bool":    return "bool"
    case "char":    return "char16_t"
    case "sbyte":   return "int8_t"
    case "byte":    return "uint8_t"
    case "short":   return "int16_t"
    case "ushort":  return "uint16_t"
    case "int":     return "int32_t"
    case "uint":    return "uint32_t"
    case "long":    return "int64_t"
    case "ulong":   return "uint64_t"
    case "float":   return "float"
    case "double":  return "double"
    case "decimal": return "double"
    case "string":  return "std::string"
    case "object":  return "void*"
    case "IntPtr":  return "intptr_t"
    case "UIntPtr": return "uintptr_t"
    case "Type":    return "void*"
    default:        return cppIdentifier(name)
    }
}

// MARK: - Referenced Type Extraction

func extractReferencedTypeNames(_ typeName: String) -> [String] {
    var result: [String] = []
    var name = typeName.trimmingCharacters(in: .whitespaces)

    while name.hasSuffix("[]") { name = String(name.dropLast(2)) }
    while name.hasSuffix("?") { name = String(name.dropLast(1)) }

    if let openIdx = findTopLevelGenericOpen(name) {
        let outer = String(name[name.startIndex..<openIdx])
        if !isBuiltinType(outer) && !outer.isEmpty {
            result.append(stripAritySuffix(outer))
        }
        let argsContent = String(name[name.index(after: openIdx)..<name.index(before: name.endIndex)])
        for arg in splitGenericArgs(argsContent) {
            result.append(contentsOf: extractReferencedTypeNames(arg.trimmingCharacters(in: .whitespaces)))
        }
    } else if !isBuiltinType(name) && !name.isEmpty {
        result.append(stripAritySuffix(name))
    }

    return result
}

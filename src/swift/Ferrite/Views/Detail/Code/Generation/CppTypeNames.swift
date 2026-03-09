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


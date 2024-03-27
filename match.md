```
// !!! do not support any comments, this is just for document 

// name: IsWindows
and {
    ua<platform> contains "windows"
}

// name: IsAndroid
and {
    ua<platform> contains "android"
}

// name: IsWindowsOrAndroid
or {
    ref<IsWindows>

    ref<IsAndroid>
}

// name: CanAcceptGzip
and {
    header<accept-encoding> any contains "gzip"
}

// 
and {
    // regexp must be a single line
    url match ^/account/(?<name>\w+)/index\.html$
    
    ref<IsWindowsOrAndroid>
    
    ref<CanAcceptGzip>
    
    // all of the query values of `servers` must in ["10", "11", "12"]
    // all multi value policies: `all`, `any`, `first`, `last`, `nth#(idx)`; `first` is the default
    // all operations: `in`, `eq`, `contains`, `match`, `gt`, `lt`, `ge`, `le`
    query<servers> all in [
        "10", "11", "12"
    ]
}
```
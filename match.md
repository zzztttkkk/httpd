```
IsWindows:and {
    ua<platform> contains "windows";
}

IsAndroid:and {
    ua<platform> contains "android";
}

IsWindowsOrAndroid:or {
    ref<IsWindows>;

    ref<IsAndroid>;
}

CanAcceptGzip:and {
    header<accept-encoding:all> contains "gzip";
}

and {
    path match "^/account/(?<name>\\w+)/index\\.html$";
    
    ref<IsWindowsOrAndroid>;
    
    ref<CanAcceptGzip>;
    
    query<servers:all> in [
        "10", "11", "12"
    ];
}
```
```
match IsWindows:and {
    ua<platform> contains "windows";
}

match IsNotWindows {
    ref<IsNotWindows> not;
}

match IsAndroid {
    ua<platform> contains "android";
}

match IsWindowsOrAndroid:or {
    ref<IsWindows>;

    ref<IsAndroid>;
}

match CanAcceptGzip {
    header<accept-encoding:any> contains "gzip";
}

match IsAccountIndex {
    path match "^/account/(?<name>\\w+)/index\\.html$";
    
    ref<IsWindowsOrAndroid>;
    
    ref<CanAcceptGzip>;
}

fn set_code(value) {
    code set ${value};
}

fn main {
    if IsNoWindows {
        set_code(404);
        return;
    }
}
```
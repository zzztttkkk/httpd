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
    header<accept-encoding:all> contains "gzip";
}

match IsAccountIndex {
    path match "^/account/(?<name>\\w+)/index\\.html$";
    
    ref<IsWindowsOrAndroid>;
    
    ref<CanAcceptGzip>;
    
    query not contains ["lang"];
}

fn return_404_if_not_windows {
    match IsNotWindows;
    code replace 404;
    end;
}

return_404_if_not_windows();



```
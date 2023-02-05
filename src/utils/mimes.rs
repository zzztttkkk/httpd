use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static COMMON_MIME_TYPES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
    // JSON.stringify(Array.from(temp1.querySelectorAll("tr")).map(ele=>{ let tds = Array.from(ele.querySelectorAll("td")); return [tds[0].innerText, tds[2].innerText]; }));
    let common_mime_types = [
		[".aac","audio/aac"],[".abw","application/x-abiword"],[".arc","application/x-freearc"],[".avif","image/avif"],
		[".avi","video/x-msvideo"],[".azw","application/vnd.amazon.ebook"],[".bin","application/octet-stream"],
		[".bmp","image/bmp"],[".bz","application/x-bzip"],[".bz2","application/x-bzip2"],[".cda","application/x-cdf"],
		[".csh","application/x-csh"],[".css","text/css"],[".csv","text/csv"],[".doc","application/msword"],
		[".docx","application/vnd.openxmlformats-officedocument.wordprocessingml.document"],[".eot","application/vnd.ms-fontobject"],
		[".epub","application/epub+zip"],[".gz","application/gzip"],[".gif","image/gif"],[".htm, .html","text/html"],
		[".ico","image/vnd.microsoft.icon"],[".ics","text/calendar"],[".jar","application/java-archive"],[".jpeg, .jpg","image/jpeg"],
		[".js","text/javascript (Specifications: HTML and RFC 9239)"],[".json","application/json"],[".jsonld","application/ld+json"],
		[".mid, .midi","audio/midi, audio/x-midi"],[".mjs","text/javascript"],[".mp3","audio/mpeg"],[".mp4","video/mp4"],[".mpeg","video/mpeg"],
		[".mpkg","application/vnd.apple.installer+xml"],[".odp","application/vnd.oasis.opendocument.presentation"],
		[".ods","application/vnd.oasis.opendocument.spreadsheet"],[".odt","application/vnd.oasis.opendocument.text"],
		[".oga","audio/ogg"],[".ogv","video/ogg"],[".ogx","application/ogg"],[".opus","audio/opus"],[".otf","font/otf"],
		[".png","image/png"],[".pdf","application/pdf"],[".php","application/x-httpd-php"],[".ppt","application/vnd.ms-powerpoint"],
		[".pptx","application/vnd.openxmlformats-officedocument.presentationml.presentation"],[".rar","application/vnd.rar"],
		[".rtf","application/rtf"],[".sh","application/x-sh"],[".svg","image/svg+xml"],[".tar","application/x-tar"],
		[".tif, .tiff","image/tiff"],[".ts","video/mp2t"],[".ttf","font/ttf"],[".txt","text/plain"],[".vsd","application/vnd.visio"],
		[".wav","audio/wav"],[".weba","audio/webm"],[".webm","video/webm"],[".webp","image/webp"],[".woff","font/woff"],
		[".woff2","font/woff2"],[".xhtml","application/xhtml+xml"],[".xls","application/vnd.ms-excel"],
		[".xlsx","application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"],
		[".xml","application/xml is recommended as of RFC 7303 (section 4.1), but text/xml is still used sometimes. You can assign a specific MIME type to a file with .xml extension depending on how its contents are meant to be interpreted. For instance, an Atom feed is application/atom+xml, but application/xml serves as a valid default."],[".xul","application/vnd.mozilla.xul+xml"],
		[".zip","application/zip"],[".3gp","video/3gpp; audio/3gpp if it doesn't contain video"],
		[".3g2","video/3gpp2; audio/3gpp2 if it doesn't contain video"],[".7z","application/x-7z-compressed"]
	];

    for mime in common_mime_types {
        let (keys, vals) = (mime[0], mime[1]);
        let val = vals
            .splitn(2, " ")
            .next()
            .unwrap()
            .splitn(2, ",")
            .next()
            .unwrap()
            .splitn(2, ";")
            .next()
            .unwrap()
            .trim();

        for mut key in keys.split(",") {
            key = &(key.trim())[1..];
            m.insert(key.to_string(), val.to_string());
        }
    }

    m.insert("rs".to_string(), "text/plain".to_string());
    m.insert("py".to_string(), "text/plain".to_string());
    m.insert("go".to_string(), "text/plain".to_string());
    m.insert("c".to_string(), "text/plain".to_string());
    m.insert("h".to_string(), "text/plain".to_string());
    m.insert("cpp".to_string(), "text/plain".to_string());
    m.insert("hpp".to_string(), "text/plain".to_string());
    m.insert("toml".to_string(), "application/toml".to_string());

    m
});

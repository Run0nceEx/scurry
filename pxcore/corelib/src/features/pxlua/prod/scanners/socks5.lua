
ENGINE["ADDONS"]["SOCKS5"] = "SOCKS5_DETECT"


function SOCKS5_DETECT(stream) { 
    stream.write({1, 2, 3, 4, 5});
    local buf = stream.read(16);
}



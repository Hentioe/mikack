document = {
  cookie: ""
};

function PrefixInteger(num, length) {
  return (Array(length).join("0") + num).slice(-length);
}

function kVZhChChOYXq(name_, value_) {
  var Days_ = 30;
  var exp_ = new Date();
  exp_.setTime(exp_.getTime() + Days_ * 24 * 60 * 60 * 1000);
  document.cookie =
    name_ + "=" + escape(value_) + ";expires=" + exp_.toGMTString();
}

function eWSqInvctSGfzolTCW(name) {
  var exp = new Date();
  exp.setTime(exp.getTime() - 1);
  var cval = XCAAqgFOhyJN(name);
  if (cval != null)
    document.cookie = name + "=" + cval + ";expires=" + exp.toGMTString();
}

function XCAAqgFOhyJN(name_) {
  var arr_,
    reg_ = new RegExp("(^| )" + name_ + "=([^;]*)(;|$)");
  if ((arr_ = document.cookie.match(reg_))) return unescape(arr_[2]);
  else return null;
}

eWSqInvctSGfzolTCW("xue");
function openimg(a, b, c, caonima_src) {
  var str = caonima_src;
  var pid = a + "/";
  var xuu = XCAAqgFOhyJN("xue");
  if (!xuu) {
    xuu = "1";
  }
  if (c == "-1") {
    if (xuu == "1") {
      console.log("已經是第一頁了");
      return null;
    }
    xuu = Number(xuu) - Number(1);
  } else if (Number(c) >= Number(2)) {
    xuu = Number(c);
  } else if (xuu == b) {
    console.log("已經是最後一頁了");
    return null;
  } else if (c == "1") {
    xuu++;
  } else {
    xuu = Number(c);
  }
  kVZhChChOYXq("xue", xuu);
  var typ = PrefixInteger(xuu, 3);
  var img = str.split(pid);
  return img[0] + pid + typ + ".jpg";
}
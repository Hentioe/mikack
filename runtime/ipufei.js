function base64decode(str) {
  var base64EncodeChars =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  var base64DecodeChars = new Array(
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    62,
    -1,
    -1,
    -1,
    63,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    61,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    26,
    27,
    28,
    29,
    30,
    31,
    32,
    33,
    34,
    35,
    36,
    37,
    38,
    39,
    40,
    41,
    42,
    43,
    44,
    45,
    46,
    47,
    48,
    49,
    50,
    51,
    -1,
    -1,
    -1,
    -1,
    -1
  );
  var c1, c2, c3, c4;
  var i, len, out;
  len = str.length;
  i = 0;
  out = "";
  while (i < len) {
    do {
      c1 = base64DecodeChars[str.charCodeAt(i++) & 255];
    } while (i < len && c1 == -1);
    if (c1 == -1) {
      break;
    }
    do {
      c2 = base64DecodeChars[str.charCodeAt(i++) & 255];
    } while (i < len && c2 == -1);
    if (c2 == -1) {
      break;
    }
    out += String.fromCharCode((c1 << 2) | ((c2 & 48) >> 4));
    do {
      c3 = str.charCodeAt(i++) & 255;
      if (c3 == 61) {
        return out;
      }
      c3 = base64DecodeChars[c3];
    } while (i < len && c3 == -1);
    if (c3 == -1) {
      break;
    }
    out += String.fromCharCode(((c2 & 15) << 4) | ((c3 & 60) >> 2));
    do {
      c4 = str.charCodeAt(i++) & 255;
      if (c4 == 61) {
        return out;
      }
      c4 = base64DecodeChars[c4];
    } while (i < len && c4 == -1);
    if (c4 == -1) {
      break;
    }
    out += String.fromCharCode(((c3 & 3) << 6) | c4);
  }
  return out;
}

function decode(packed) {
  photosr = [];
  eval(eval(base64decode(packed).slice(4)));
  return photosr.filter(p => p).map(p => `http://res.img.220012.net/${p}`);
}

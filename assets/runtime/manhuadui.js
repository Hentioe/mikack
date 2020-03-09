function decrypt20180904(chapterImages) {
  var key = CryptoJS.enc.Utf8.parse("123456781234567G"); //十六位字符作为密钥
  var iv = CryptoJS.enc.Utf8.parse("ABCDEF1G34123412");
  var decrypt = CryptoJS.AES.decrypt(chapterImages, key, {
    iv: iv,
    mode: CryptoJS.mode.CBC,
    padding: CryptoJS.pad.Pkcs7
  });
  var decryptedStr = decrypt.toString(CryptoJS.enc.Utf8);
  return JSON.parse(decryptedStr.toString());
}

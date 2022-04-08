function handleAuthError(data, href) {
  if (data['success'] == false) {
    if (data['errcode'] == 101) {
      window.location.href = href;
      return true;
    }
    return false;
  }
}

function handleError(data) {
  if (data['success'] == true) {
    return false;
  }
  const msg = data['message'] + '\n' + data['reason'];
  alert(msg);
  return true;
}

function display_ct() {
  var x = new Date()
  var x1 = x.getFullYear() + "/" + x.getMonth()+ "/" + x.getDate()
  x1 = x1 + " - " +  x.getHours( )+ ":" +  x.getMinutes() + ":" +  x.getSeconds();
  document.getElementById('ct').style.fontSize="-1";
  document.getElementById('ct').innerHTML = x1;
  display_c();
}

function display_c(){
  var refresh=1000;
  mytime=setTimeout('display_ct()',refresh)
}

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
  const msg = data['message'] + '(' + data['errcode'] + ')' + '\n' + data['reason'];
  alert(msg);
  return true;
}

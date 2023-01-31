function isAuthError(data) {
  if (data["success"] == false) {
    if (data["errcode"] == 101) {
      return true;
    }
    return false;
  }
}

function handleAuthError(data) {
  if (isAuthError(data)) {
    window.location.href = '/login/';
    return true;
  }
  return false;
}

function handleError(data) {
  if (data["success"] == true) {
    return false;
  }
  const msg =
    data["message"] + "(" + data["errcode"] + ")" + "\n" + data["reason"];
  alert(msg);
  return true;
}

function postPasswordForm(formId, passwordId, configmPasswordId, minLength) {
  e.preventDefault();

  if (!jQuery || !jQuery.fn.ajaxForm) {
    console.error("jQuery or ajaxForm plugin is not loaded");
    return false;
  }
  
  const newPassword = document.getElementById(passwordId).value;
  const confirmPassword = document.getElementById(configmPasswordId).value;

  // Validate the form inputs
  if (newPassword !== confirmPassword) {
    alert("入力した２つのパスワードが一致しません");
    return false;
  }

  if (newPassword.length < minLength) {
    alert("パスワードは8文字以上必要です");
    return false;
  }

  $("#" + formId)
    .ajaxForm()
    .submit();

  return false;
}

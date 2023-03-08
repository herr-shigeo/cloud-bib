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
    window.location.href = "/login/";
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

function handleRedirect(data) {
  if (isAuthError(data)) {
    window.location.href = "/login/";
  } else if (
    data.redirect_to !== undefined &&
    data.redirect_to !== null &&
    data.redirect_to !== ""
  ) {
    window.location.href = data.redirect_to;
  }
}

function checkPassword(uname, password, confirmPassword, minLength) {
  // Validate the form inputs
  if (password !== confirmPassword) {
    alert(`入力した２つのパスワードが一致しません`);
    return false;
  }

  if (password.length < minLength) {
    alert(`パスワードは${minLength}文字以上必要です`);
    return false;
  }

  return true;
}

// Updates the user's password using a PUT request
function updatePassword(
  event,
  url,
  formId,
  unameId,
  passwordId,
  confirmPasswordId,
  minLength
) {
  const form = document.getElementById(formId);
  const uname = form.elements[unameId].value;
  const password = form.elements[passwordId].value;
  const confirmPassword = form.elements[confirmPasswordId].value;

  // Prevent the default form submission
  event.preventDefault();

  if (!checkPassword(name, password, confirmPassword, minLength)) {
    return false;
  }

  const data = {
    uname,
    password,
    confirm_password: confirmPassword,
  };

  // Send the PUT request using fetchData
  fetchData("PUT", JSON.stringify(data), url, "変更しました", null, {
    "Content-Type": "application/json",
  });

  // Return false to prevent the form from submitting normally
  return false;
}

// Submits a json data converting from a form using fetch
// formId: ID of the form to submit
// action: HTTP method to use (e.g. 'POST')
// url: URL to send the request to
// okMessage: Message to display if the request is successful
function fetchJsonData(event, formId, action, url, okMessage, callback) {
  const form = document.getElementById(formId);
  const formData = new FormData(form);

  // Convert form data to JSON object
  const formDataObj = {};
  for (const [key, value] of formData.entries()) {
    if (!key.startsWith("_")) {
      formDataObj[key] = value;
    }
  }
  const jsonData = JSON.stringify(formDataObj);

  // Prevent the default form submission
  event.preventDefault();

  // Send the request using fetchData
  fetchData(action, jsonData, url, okMessage, callback, {
    "Content-Type": "application/json",
  });

  // Return false to prevent the form from submitting normally
  return false;
}

// Sends a fetch request
// action: HTTP method to use (e.g. 'POST')
// body: Request body
// url: URL to send the request to
// okMessage: Message to display if the request is successful
// contentType: Content-Type header value to set
function fetchData(action, body, url, okMessage, callback, contentType) {
  return fetch(url, {
    method: action,
    body: body,
    headers: contentType,
  })
    .then((response) => response.json())
    .then((data) => {
      if (callback !== null) {
        callback(data);
      } else if (!handleError(data)) {
        if (okMessage !== null) {
          alert(okMessage);
        }
        handleRedirect(data);
      }
    })
    .catch((error) => {
      console.error("There was a problem with the fetch operation:", error);
    });
}

function sendLogout() {
  fetch("/auth/logout", {
    method: "POST",
  })
    .then((response) => {
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
    })
    .then(() => {
      window.location.href = "/login/";
    })
    .catch((error) => {
      console.error("There was a problem with the fetch operation:", error);
    });
}

function constructUrlFromForm(formId, baseUrl) {
  const form = document.getElementById(formId);
  const inputs = form.elements;
  let queryString = "";
  let isFirstParam = true;

  for (let i = 0; i < inputs.length; i++) {
    const input = inputs[i];
    const name = encodeURIComponent(input.name);
    const value = encodeURIComponent(input.value);

    if (name && value !== null && value !== undefined) {
      if (isFirstParam) {
        if (baseUrl.includes("?")) {
          queryString += "&";
        } else {
          queryString += "?";
        }
        isFirstParam = false;
      } else {
        queryString += "&";
      }
      queryString += name + "=" + value;
    }
  }

  return baseUrl + queryString;
}

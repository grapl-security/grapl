const engagement_edge = "https://jzfee2ecp8.execute-api.us-east-1.amazonaws.com/prod/";

if (engagement_edge.length === 0) {
    console.assert("Engagement Edge URL can not be empty. Run build.sh");
}

async function sha256(message) {
    // encode as UTF-8
    const msgBuffer = new TextEncoder('utf-8').encode(message);

    // hash the message
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);

    // convert ArrayBuffer to Array
    const hashArray = Array.from(new Uint8Array(hashBuffer));

    // convert bytes to hex string
    return hashArray.map(b => ('00' + b.toString(16)).slice(-2)).join('');
}


const sha256WithPepper = async (message) => {
    // The pepper only exists to prevent rainbow tables for extremely weak passwords
    // Client side hashing itself is only to prevent cases where the password is
    // exposed before it makes it into the password database
    const pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254";
    let hashed = await sha256(message + pepper);

    for (let i = 0; i < 5000; i++) {
        hashed = await sha256(hashed)
    }
    return hashed
};


const checkLogin = async () => {
    const res = await fetch(`${engagement_edge}checkLogin`, {
        method: 'get',
    });

    const body = await res.json();

    return body === 'True';
};

const login = async (username, password) => {
    const res = await fetch(`${engagement_edge}login`, {
        method: 'post',
        body: JSON.stringify({
            'username': username,
            'password': password
        })
    });

    const body = await res.json();

    return body === 'True';
};

document.addEventListener('DOMContentLoaded', async (event) => {
    if (await checkLogin()) {
        // Redirect to lenses.html if we have a valid JWT
        window.location.href = 'lenses.html';
    } else {
        $('#submitbtn').click(async (submit) => {
            const username = $("#uname").val();
            const password = await sha256WithPepper($("#psw").val());
            console.log(`logging in with password: ${password}`);
            const succ = await login(username, password);
            console.log(`login success ${succ}`)
            // window.location.href = 'lenses.html';
        })
    }
});
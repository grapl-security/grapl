async function sha256(message: string) {
    // encode as UTF-8
    const msgBuffer = new TextEncoder().encode(message);

    // hash the message
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);

    // convert ArrayBuffer to Array
    const hashArray = Array.from(new Uint8Array(hashBuffer));

    // convert bytes to hex string
    return hashArray.map(b => ('00' + b.toString(16)).slice(-2)).join('');
}


export const sha256WithPepper = async (username: string, password: string) => {
    // The pepper only exists to prevent rainbow tables for extremely weak passwords
    // Client side hashing itself is only to prevent cases where the password is
    // exposed before it makes it into the password database
    const pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254";

    let hashed = await sha256(password + pepper + username);

    for (let i = 0; i < 5000; i++) {
        hashed = await sha256(hashed)
    }

    return hashed
};
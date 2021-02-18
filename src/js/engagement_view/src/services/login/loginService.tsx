import {apiFetchWithBody} from '../fetch';

export const loginService = async (username: string, password: string) => {
    const loginBody = JSON.stringify({ 'username': username, 'password': password });

    try {
        const loginData = await apiFetchWithBody(`prod/auth/login`, "post", loginBody)
        return loginData['success'] === 'True';
    } catch (e) {
        console.log("Login Error", e);
        return false;
    }
};

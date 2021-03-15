import DEV_API_EDGES from '../constants';
import {apiPostRequestWithBody} from '../fetch';

export const loginService = async (username: string, password: string) => {
    const loginBody = JSON.stringify({ 'username': username, 'password': password });

    try {
        const loginData = await apiPostRequestWithBody(`${DEV_API_EDGES.auth}/login`, "post", loginBody)
        return loginData['success'] === 'True';
    } catch (e) {
        console.log("Login Error", e);
        return false;
    }
};
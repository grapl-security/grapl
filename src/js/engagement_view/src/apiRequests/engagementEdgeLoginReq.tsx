import { getAuthEdge } from '../services/getApiURLs';

export const engagementEdgeLoginReq = async (username: string, password: string) => {
    const engagement_edge = getAuthEdge();

    try {
        const res = await fetch(`${engagement_edge}login`, {
            method: 'post',
            body: JSON.stringify({
                'username': username,
                'password': password
            }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        });
        
        const body = await res.json();

        return body['success'] === 'True';
    } catch (e) {
        console.log("Login Error", e);
        return false;
    }
};
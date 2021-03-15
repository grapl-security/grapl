import { apiFetchWithBody } from "../fetch";
import DEV_API_EDGES from "../constants";

export const checkLogin = async (): Promise<boolean | null> => {
    try {
        const loginData = await apiFetchWithBody(`${DEV_API_EDGES.auth}/checkLogin`, "post");
        return loginData['success'] === 'True';
    } catch (e) {
        return null;
    }
};

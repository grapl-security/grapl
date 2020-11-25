import { getEngagementEdge } from "../modules/GraphViz/engagement_edge/getApiURLs";

const notebookEdge = getEngagementEdge();


export const getNotebookUrl = async (): Promise<boolean | null> => {
    const res = await fetch(
        `${notebookEdge}getNotebook`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        })
        .then(res => res.json())
        .then(res => {
            console.log("Res", res)
            if (res.errors) {
                console.error("Unable to retrieve Sagemaker url", res.errors);            }
            return res
        })

        const sagemakerUrl = res.success.notebook_url;

        window.open(sagemakerUrl); 
        
        return true; 
};

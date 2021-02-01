export const apiFetch = async (urlSlug: string, method = "GET") => {
  const response = await fetch(urlSlug, {
    method,
    credentials: "include",
    headers: new Headers({
      "Content-Type": "application/json",
    }),
  }).catch((e) => {
    console.warn(e);
    throw new Error(`API Request Error: ${e}`);
  });

  return response.json();
};


export const apiFetchWithBody = async(urlSlug: string, method = "POST", body: string) => {
  const response = await fetch(urlSlug, {
    method,
    credentials: "include",
    headers: new Headers({
      "Content-Type": "application/json",
    }),
    body: body,
  }).catch((e) => {
    console.warn(e);
    throw new Error(`Error with Post Request: ${e}`);
  });

  return response.json();
};

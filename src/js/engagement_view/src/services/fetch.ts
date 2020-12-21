const apiFetch = async (urlSlug: string, method = "get") => {
  const url = `/${encodeURIComponent(urlSlug)}`;

  const response = await fetch(url, {
    method,
    credentials: "include",
    headers: new Headers({
      "Content-Type": "application/json",
    }),
  }).catch((e) => {
    console.warn(e);
    throw new Error(`Error: ${e}`);
  });

  return response.json();
};

export default apiFetch;

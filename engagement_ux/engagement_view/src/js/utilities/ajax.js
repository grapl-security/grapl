export const get = async (url) => {
  const response = await fetch(url, {
    method: 'GET',
  });

  if (!response.ok) throw response;

  return response.json();
};

export const post = async (url, body) => {
  const response = await fetch(url, {
    method: 'POST',
    body: JSON.stringify(body),
  });

  if (!response.ok) throw response;

  return response.json();
};

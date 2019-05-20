import requests

url = "https://dzuo4ozud3.execute-api.us-east-1.amazonaws.com/"

res = requests.get(url, data={
    "engagement_id": "foo",
    "uid_hashes": []
})

print(res)
print(res.__dict__)

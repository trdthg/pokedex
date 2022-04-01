#
import requests

res = requests.get("https://api.airtable.com/v0/meta/bases", headers={
    "Authorization": "Bearer key3qwcREeUm8u8QE"
})
print(res.status_code)

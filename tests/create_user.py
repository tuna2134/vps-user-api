import requests


base_url = "http://localhost:3000"

def test_create_user():
    url = f"{base_url}/users"
    data = {
        "username": "testuser",
        "email": "test@example.com",
    }
    response = requests.post(url, json=data)
    assert response.status_code == 200, f"Expected status code 200, got {response.status_code}"
    response_data = response.json()
    assert "token" in response_data, "Response should contain a token"
    print(f"User created successfully: {response_data}")
    return response_data["token"]


def test_register_user(code: str, token: str) -> None:
    data = {
        "token": token,
        "code": code,
        "password": "abcd1234",
    }
    response = requests.post(base_url + "/users/register", json=data)
    assert response.status_code == 200, f"Expected status code 200, got {response.status_code}"
    print("User registered successfully:", response.json())


token = test_create_user()
test_register_user(input("code: "), token)
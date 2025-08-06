docker run --name test-db -p 5432:5432 -e POSTGRES_PASSWORD=postgres -d postgres
docker run --name test-redis -p 6379:6379 -d redis
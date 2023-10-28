postgres_up:
	@echo "Starting Postgres & Redis database"
	bash ./scripts/init_db.sh
	@echo "Done"

redis_up:
	@echo "Starting Postgres & Redis database"
	bash ./scripts/init_redis.sh
	@echo "Done"

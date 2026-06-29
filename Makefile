.PHONY: backend/seed
backend/seed:
	cd apps/backend && cargo run --bin seed

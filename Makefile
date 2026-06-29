.PHONY: backend/seed
backend/seed:
	cd apps/backend && cargo run --bin seed

.PHONY: backend/start
backend/start:
	cd apps/backend && cargo run

.PHONY: backend/fmt
backend/fmt:
	cd apps/backend && cargo fmt

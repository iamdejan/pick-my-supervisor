.PHONY: backend/seed
backend/seed:
	cd apps/backend && cargo run --bin seed

.PHONY: backend/fmt
backend/fmt:
	cd apps/backend && cargo fmt

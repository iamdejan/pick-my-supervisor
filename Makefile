.PHONY: backend/seed
backend/seed:
	cd apps/backend && cargo run --bin seed

.PHONY: backend/start
backend/start:
	cd apps/backend && cargo run

.PHONY: backend/fmt
backend/fmt:
	cd apps/backend && cargo fmt

.PHONY: frontend/lint
frontend/lint:
	cd apps/frontend && pnpm run lint

.PHONY: frontend/fmt
frontend/fmt:
	cd apps/frontend && pnpm run lint:fix

.PHONY: frontend/build
frontend/build:
	cd apps/frontend && pnpm run build

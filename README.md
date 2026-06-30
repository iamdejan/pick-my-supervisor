# Pick My Supervisor

A simple full-stack web application to pick Universiti Malaya's supervisor based on our interest.

## Prerequisites

Before running this project, ensure you have the following installed:
1. **Node.js** (v24 or higher)
2. **PNPM** - Package manager
    ```bash
    npm install -g pnpm
    ```
3. **Rust programming language**, installed using [rustup](https://rustup.rs/).

## Getting Started

1. **Install dependencies:**
    Frontend:
    ```bash
    pnpm run frontend:deps
    ```

    Backend:
    ```bash
    pnpm run backend:build
    ```

2. **Copy and adjust environment variables:**
    In each `backend` and `frontend` packages, there is `.env.example` package. Copy to `.env`, then adjust the values as needed.

3. **Start the development server:**
    Backend:
    ```bash
    pnpm run backend:start
    ```

     Frontend (must wait until the backend starts):
     ```bash
     pnpm run frontend:start
     ```

## Cloud Run Deployment (Backend)

The backend is deployed to Google Cloud Run using source-based deployment.

### Prerequisites

1. **Google Cloud SDK** (`gcloud`) installed and authenticated.
2. **Secrets stored in Google Cloud Secret Manager:**

    ```bash
    echo -n "<qdrant-api-key>" | gcloud secrets create qdrant-api-key --data-file=-
    echo -n "<openrouter-api-key>" | gcloud secrets create openrouter-api-key --data-file=-
    ```
3.  **Allow secrets to be accessed by Service Account:**

    ```bash
    gcloud secrets add-iam-policy-binding qdrant-api-key \
        --member="serviceAccount:{{service account email}}" \
        --role="roles/secretmanager.secretAccessor"
    ```

    ```bash
    gcloud secrets add-iam-policy-binding openrouter-api-key \
        --member="serviceAccount:{{service account email}}" \
        --role="roles/secretmanager.secretAccessor"
    ```

### Files

| File | Purpose |
|---|---|
| `apps/backend/Dockerfile` | Container image — builds the Rust binary and runs it on port 8080 |
| `apps/backend/.env.cloudrun.yaml` | Non-sensitive env vars (`HOST`, `QDRANT_CLUSTER_ENDPOINT`) — gitignored |
| `apps/backend/.env.example` | Template for local `.env` setup |

### Deploy

```bash
gcloud run deploy pick-my-supervisor \
    --source ./apps/backend \
    --region asia-southeast1 \
    --allow-unauthenticated \
    --env-vars-file apps/backend/.env.cloudrun.yaml \
    --set-secrets="QDRANT_API_KEY=qdrant-api-key:latest,OPENROUTER_API_KEY=openrouter-api-key:latest"
```

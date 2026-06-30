# Plan: Cloud Run Environment Variables Setup

## Problem

The backend needs environment variables (Qdrant credentials, OpenRouter API key, host/port) to run on Cloud Run. The current deployment command doesn't pass any.

## Steps

### 1. ✅ Fix Dockerfile (`apps/backend/Dockerfile`)

- Changed `CMD` binary path from `./target/release/cloud-rust` to `./target/release/main`
- Added `ENV HOST=0.0.0.0` so Cloud Run binds to the correct interface (Cloud Run auto-sets `PORT=8080`, no need to set that)

### 2. ✅ Create `apps/backend/.env.cloudrun.yaml`

Contains non-sensitive vars:
```yaml
HOST: "0.0.0.0"
QDRANT_CLUSTER_ENDPOINT: "https://c25d38ff-4fd6-4e20-9aa2-372784003d53.australia-southeast1-0.gcp.cloud.qdrant.io:6334"
```

Note: This file is gitignored (added to `.gitignore`).

### 3. Store secrets in Google Cloud Secret Manager

Run these commands once to create the secrets:

```bash
# Store Qdrant API key
echo -n "<your-qdrant-api-key>" | gcloud secrets create qdrant-api-key --data-file=-

# Store OpenRouter API key
echo -n "<your-openrouter-api-key>" | gcloud secrets create openrouter-api-key --data-file=-
```

To update existing secrets later:

```bash
echo -n "<new-value>" | gcloud secrets versions add qdrant-api-key --data-file=-
echo -n "<new-value>" | gcloud secrets versions add openrouter-api-key --data-file=-
```

### 4. Final deploy command

```bash
gcloud run deploy pick-my-supervisor \
    --source ./apps/backend \
    --region asia-southeast1 \
    --allow-unauthenticated \
    --env-vars-file apps/backend/.env.cloudrun.yaml \
    --set-secrets="QDRANT_API_KEY=qdrant-api-key:latest,OPENROUTER_API_KEY=openrouter-api-key:latest"
```

### 5. Validation

- After deployment, hit the `/healthcheck` endpoint to verify the service is running
- Test `/supervisors/pick` with a sample request to ensure Qdrant and OpenRouter connectivity works

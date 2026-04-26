#!/bin/bash
# deploy-api.sh
ORG_ID="1032448450426"
PROJECT_ID="rustic-ai-rkapps"
RUSTIC_AI_FIREBASE_PROJECT_ID=$PROJECT_ID
REGION="us-central1"
IMAGE_REGISTRY="us-central1-docker.pkg.dev/$PROJECT_ID"
COMPUTE_SA_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
COMPUTE_SA="$COMPUTE_SA_NUMBER-compute@developer.gserviceaccount.com"
GCS_BUCKET="$PROJECT_ID-data"
APP_CONFIG_PATH="gs://$GCS_BUCKET/app_config.json"
TEMPLATES_PATH="gs://$GCS_BUCKET/templates.json"

docker build --no-cache -f Dockerfile.api \
  -t $IMAGE_REGISTRY/rustic-ai-api/rustic-ai-api . \
  && docker push $IMAGE_REGISTRY/rustic-ai-api/rustic-ai-api \
  && gcloud run deploy rustic-ai-api \
        --image $IMAGE_REGISTRY/rustic-ai-api/rustic-ai-api \
        --region us-central1 \
        --allow-unauthenticated \
        --set-env-vars APP_CONFIG_PATH=$APP_CONFIG_PATH \
        --set-env-vars TEMPLATES_PATH=$TEMPLATES_PATH \
        --set-env-vars MONGO_ATLAS_CONN_STR=$MONGO_ATLAS_CONN_STR \
        --set-env-vars GCP_LLM_BASE_URL=$GCP_LLM_BASE_URL \
        --set-env-vars RUSTIC_AI_FIREBASE_PROJECT_ID=$PROJECT_ID \
        --set-env-vars OPENAI_API_KEY=$OPENAI_API_KEY \
        --set-env-vars GEMINI_API_KEY=$GEMINI_API_KEY \
        --set-env-vars ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
        --set-env-vars "^|^RUST_LOG=$RUST_LOG_VALUE" \
        --set-env-vars LOG_FORMAT=json


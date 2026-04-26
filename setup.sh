ORG_ID="1032448450426"
PROJECT_ID="rustic-ai-rkapps"
RUSTIC_AI_FIREBASE_PROJECT_ID=$PROJECT_ID
REGION="us-central1"
IMAGE_REGISTRY="us-central1-docker.pkg.dev/$PROJECT_ID"
SCHEDULER_SA="cloud-scheduler-sa@$PROJECT_ID.iam.gserviceaccount.com"

RUST_LOG_VALUE="rustic_ai_services=info,agentic_core=info,rustic_ai_api=info"
echo "Compute SA: $COMPUTE_SA"

gcloud auth login

# Create project only if it doesn't exist
if gcloud projects describe $PROJECT_ID &>/dev/null; then
  echo "Project $PROJECT_ID already exists, skipping creation..."
else
  echo "Creating project $PROJECT_ID..."
  gcloud projects create $PROJECT_ID \
    --name "Rustic AI Demo" \
    --organization $ORG_ID
fi

# Set active project
gcloud config set project $PROJECT_ID

# Pause for billing setup
BILLING=$(gcloud billing projects describe $PROJECT_ID --format="value(billingEnabled)")
if [ "$BILLING" != "True" ]; then
  echo "==========================================="
  echo "ACTION REQUIRED: Enable billing for project $PROJECT_ID"
  echo "1. Go to https://console.cloud.google.com/billing/projects"
  echo "2. Link a billing account to $PROJECT_ID"
  echo "3. Press ENTER when done..."
  echo "==========================================="
  read -p ""

  while true; do
    BILLING=$(gcloud billing projects describe $PROJECT_ID --format="value(billingEnabled)")
    if [ "$BILLING" = "True" ]; then
      echo "Billing confirmed active, continuing..."
      break
    else
      echo "Billing not yet active, waiting 10 seconds..."
      sleep 10
    fi
  done
else
  echo "Billing already active, skipping..."
fi


#cd /media/raghu/data2/Workspace/Projects/fin-tracker-backend-rs
COMPUTE_SA_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
COMPUTE_SA="$COMPUTE_SA_NUMBER-compute@developer.gserviceaccount.com"



# Enable APIs (safe to run multiple times)
gcloud services enable \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com \
  cloudscheduler.googleapis.com \
  secretmanager.googleapis.com \
  iam.googleapis.com \
  storage.googleapis.com

echo "Waiting for APIs to propagate..."

# Grant roles (safe to run multiple times - adds if not exists)
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$COMPUTE_SA" \
  --role="roles/editor"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$COMPUTE_SA" \
  --role="roles/run.invoker"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$COMPUTE_SA" \
  --role="roles/secretmanager.secretAccessor"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$COMPUTE_SA" \
  --role="roles/firebase.admin"

# Create Cloud Scheduler service account (skip if exists)
gcloud iam service-accounts create cloud-scheduler-sa \
  --display-name "Cloud Scheduler Service Account" \
  2>/dev/null || echo "cloud-scheduler-sa already exists, skipping..."

# Grant scheduler service account roles (safe to run multiple times)
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:cloud-scheduler-sa@$PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/run.invoker"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:cloud-scheduler-sa@$PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/run.admin"

# Create GCS bucket (skip if exists)
gcloud storage buckets create gs://$PROJECT_ID-data \
  --location $REGION \
  --project $PROJECT_ID \
  --quiet 2>/dev/null || echo "Bucket $PROJECT_ID-data already exists, skipping..."

# Grant compute service account access to bucket
gcloud storage buckets add-iam-policy-binding gs://$PROJECT_ID-data \
  --member="serviceAccount:$COMPUTE_SA" \
  --role="roles/storage.objectViewer"  


# ✅ Check first, create only if missing, fail loudly if creation fails
echo "Checking Artifact Registry repository..."
if ! gcloud artifacts repositories describe rustic-ai-api \
  --location=us-central1 \
  --project=$PROJECT_ID &>/dev/null; then

  echo "Creating repository rustic-ai-api..."
  gcloud artifacts repositories create rustic-ai-api \
    --repository-format=docker \
    --location=us-central1 \
    --project=$PROJECT_ID
  echo "✅ Repository created"
else
  echo "✅ Repository already exists"
fi

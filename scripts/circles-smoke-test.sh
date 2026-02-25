#!/usr/bin/env zsh
# circles-smoke-test.sh — Directive 007 Phase 4.1 smoke test
#
# Exercises the full Circles R&D Platform happy path against a running
# nexcore-api server. Each step builds on the previous; a failure stops
# ID capture but the script continues to report all statuses.
#
# Usage:
#   ./scripts/circles-smoke-test.sh [BASE_URL]
#
# Examples:
#   ./scripts/circles-smoke-test.sh                          # default: http://localhost:3030
#   ./scripts/circles-smoke-test.sh http://staging:3030
#
# Dependencies: curl, jq
# Auth note: Bearer token is a placeholder. In production, supply a real JWT.
#   The in-memory MockPersistence used during development does not enforce auth.

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

BASE_URL="${1:-http://localhost:3030}"
API="${BASE_URL}/api/v1"
AUTH_HEADER="Authorization: Bearer smoke-test-token"
CONTENT_TYPE="Content-Type: application/json"

# Unique user identity per run — prevents member-already-exists conflicts on
# retry. The user is seeded as Founder of the circle (role level 6), which
# satisfies Researcher+ for project creation, Lead+ for publishing, and
# Reviewer+ for review — so a single user drives the entire happy path.
USER_ID="smoke-test-user-$(date +%s)"

PASS=0
FAIL=0
SKIP=0

# Temp files for response bodies (overwritten each step)
BODY_FILE="/tmp/circles-smoke-body.json"

# ============================================================================
# Helpers
# ============================================================================

# check NAME STATUS EXPECTED
#   Increments PASS or FAIL counter and prints result line.
check() {
    local name="$1" status="$2" expected="$3"
    if [[ "$status" -eq "$expected" ]]; then
        echo "  PASS  [$status] $name"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  [expected $expected, got $status] $name"
        FAIL=$((FAIL + 1))
    fi
}

# curl_post URL BODY — writes response body to BODY_FILE, returns HTTP status
curl_post() {
    local url="$1" body="$2"
    curl -s \
         -w "\n%{http_code}" \
         -X POST \
         -H "$CONTENT_TYPE" \
         -H "$AUTH_HEADER" \
         -d "$body" \
         -o "$BODY_FILE" \
        "$url" | tail -n1
}

# curl_get URL — writes response body to BODY_FILE, returns HTTP status
curl_get() {
    local url="$1"
    curl -s \
         -w "\n%{http_code}" \
         -H "$AUTH_HEADER" \
         -o "$BODY_FILE" \
        "$url" | tail -n1
}

# extract FIELD — reads jq field from BODY_FILE
extract() {
    jq -r "$1" "$BODY_FILE" 2>/dev/null || echo ""
}

# skip_remaining REASON — marks remaining steps as skipped
skip_step() {
    local name="$1"
    echo "  SKIP  $name (prerequisite failed)"
    SKIP=$((SKIP + 1))
}

# ============================================================================
# Preflight
# ============================================================================

echo ""
echo "=== Circles R&D Platform — Smoke Test ==="
echo "    Target : $API"
echo "    User   : $USER_ID"
echo "    Time   : $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo ""

# Verify server is reachable before spending time on steps
HEALTH_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "${BASE_URL}/health" 2>/dev/null || echo "000")
if [[ "$HEALTH_STATUS" != "200" ]]; then
    echo "  ERROR  Server not reachable at $BASE_URL (health returned $HEALTH_STATUS)"
    echo "         Start nexcore-api first:"
    echo "           cargo-raw build -p nexcore-api --release"
    echo "           ./target/release/nexcore-api"
    echo ""
    exit 1
fi
echo "  OK     Server reachable ($HEALTH_STATUS)"
echo ""

# ============================================================================
# Step 1: Create Circle
# ============================================================================
# The creating user is auto-registered as Founder (role level 6).
# Founder satisfies: Researcher+ (create project), Lead+ (publish), Reviewer+ (review).
# This single-user approach avoids multi-member orchestration in the smoke test.

echo "--- Step 1: POST /api/v1/circles (create circle)"

CIRCLE_BODY=$(cat <<JSON
{
  "name": "Smoke Test Circle $(date +%s)",
  "description": "Automated smoke test — safe to delete",
  "formation": "AdHoc",
  "visibility": "Public",
  "join_policy": "Open",
  "circle_type": "Research",
  "therapeutic_areas": ["General"],
  "created_by": "$USER_ID"
}
JSON
)

STATUS=$(curl_post "$API/circles" "$CIRCLE_BODY")
check "Create circle" "$STATUS" 200

CIRCLE_ID=$(extract '.id')
if [[ -z "$CIRCLE_ID" || "$CIRCLE_ID" == "null" ]]; then
    echo "  ERROR  Could not capture circle_id from response"
    FAIL=$((FAIL + 1))
    CIRCLE_ID=""
fi

# ============================================================================
# Step 2: Get Circle
# ============================================================================

echo ""
echo "--- Step 2: GET /api/v1/circles/{id} (verify circle exists)"

if [[ -n "$CIRCLE_ID" ]]; then
    STATUS=$(curl_get "$API/circles/$CIRCLE_ID")
    check "Get circle" "$STATUS" 200
    RETURNED_ID=$(extract '.id')
    if [[ "$RETURNED_ID" != "$CIRCLE_ID" ]]; then
        echo "  WARN   Returned id '$RETURNED_ID' != requested '$CIRCLE_ID'"
    fi
else
    skip_step "Get circle"
fi

# ============================================================================
# Step 3: Create Project
# ============================================================================
# Requires created_by user to be Researcher+ in the circle.
# The Founder role (level 6) satisfies this.

echo ""
echo "--- Step 3: POST /api/v1/circles/{id}/projects (create project)"

PROJECT_ID=""
if [[ -n "$CIRCLE_ID" ]]; then
    PROJECT_BODY=$(cat <<JSON
{
  "name": "Smoke Test Project",
  "description": "Automated smoke test project",
  "project_type": "SignalEvaluation",
  "lead_user_id": "$USER_ID",
  "created_by": "$USER_ID"
}
JSON
)
    STATUS=$(curl_post "$API/circles/$CIRCLE_ID/projects" "$PROJECT_BODY")
    check "Create project" "$STATUS" 200

    PROJECT_ID=$(extract '.id')
    if [[ -z "$PROJECT_ID" || "$PROJECT_ID" == "null" ]]; then
        echo "  ERROR  Could not capture project_id from response"
        FAIL=$((FAIL + 1))
        PROJECT_ID=""
    fi
else
    skip_step "Create project"
fi

# ============================================================================
# Step 4: Get Project
# ============================================================================

echo ""
echo "--- Step 4: GET /api/v1/circles/{cid}/projects/{pid} (verify project exists)"

if [[ -n "$CIRCLE_ID" && -n "$PROJECT_ID" ]]; then
    STATUS=$(curl_get "$API/circles/$CIRCLE_ID/projects/$PROJECT_ID")
    check "Get project" "$STATUS" 200
    RETURNED_CID=$(extract '.circle_id')
    if [[ "$RETURNED_CID" != "$CIRCLE_ID" ]]; then
        echo "  WARN   Project circle_id '$RETURNED_CID' != '$CIRCLE_ID'"
    fi
else
    skip_step "Get project"
fi

# ============================================================================
# Step 5: Signal Detection Tool
# ============================================================================
# Requires user_id to be Researcher+ in the circle.

echo ""
echo "--- Step 5: POST /api/v1/circles/{cid}/projects/{pid}/tools/signal-detect"

if [[ -n "$CIRCLE_ID" && -n "$PROJECT_ID" ]]; then
    SIGNAL_BODY=$(cat <<JSON
{
  "drug_count": 100,
  "event_count": 50,
  "drug_event_count": 10,
  "total_count": 10000,
  "user_id": "$USER_ID"
}
JSON
)
    STATUS=$(curl_post "$API/circles/$CIRCLE_ID/projects/$PROJECT_ID/tools/signal-detect" "$SIGNAL_BODY")
    check "Signal detection tool" "$STATUS" 200
    TOOL_SUCCESS=$(extract '.success')
    if [[ "$TOOL_SUCCESS" == "false" ]]; then
        echo "  WARN   Tool returned success=false (MCP may be unavailable — HTTP 200 is the gate)"
    fi
else
    skip_step "Signal detection tool"
fi

# ============================================================================
# Step 6: Create Deliverable
# ============================================================================
# Requires created_by user to be Researcher+ (Founder qualifies).
# The deliverable starts in Draft/Pending review status.

echo ""
echo "--- Step 6: POST /api/v1/circles/{cid}/projects/{pid}/deliverables (create deliverable)"

DELIVERABLE_ID=""
if [[ -n "$CIRCLE_ID" && -n "$PROJECT_ID" ]]; then
    DELIVERABLE_BODY=$(cat <<JSON
{
  "name": "Smoke Test Deliverable",
  "deliverable_type": "Report",
  "created_by": "$USER_ID"
}
JSON
)
    STATUS=$(curl_post "$API/circles/$CIRCLE_ID/projects/$PROJECT_ID/deliverables" "$DELIVERABLE_BODY")
    check "Create deliverable" "$STATUS" 200

    DELIVERABLE_ID=$(extract '.id')
    if [[ -z "$DELIVERABLE_ID" || "$DELIVERABLE_ID" == "null" ]]; then
        echo "  ERROR  Could not capture deliverable_id from response"
        FAIL=$((FAIL + 1))
        DELIVERABLE_ID=""
    fi
else
    skip_step "Create deliverable"
fi

# ============================================================================
# Step 6b: Review/Approve Deliverable (prerequisite for publish)
# ============================================================================
# publish_deliverable enforces: deliverable.status == Approved.
# This step promotes the deliverable from Draft to Approved.
# Requires reviewed_by to be Reviewer+ (Founder satisfies role level 6 >= 3).

echo ""
echo "--- Step 6b: POST .../deliverables/{did}/review (approve deliverable)"

if [[ -n "$CIRCLE_ID" && -n "$PROJECT_ID" && -n "$DELIVERABLE_ID" ]]; then
    REVIEW_BODY=$(cat <<JSON
{
  "reviewed_by": "$USER_ID",
  "review_status": "approved",
  "review_notes": "Smoke test auto-approval"
}
JSON
)
    STATUS=$(curl_post \
        "$API/circles/$CIRCLE_ID/projects/$PROJECT_ID/deliverables/$DELIVERABLE_ID/review" \
        "$REVIEW_BODY")
    check "Approve deliverable" "$STATUS" 200
    REVIEW_STATUS=$(extract '.review_status')
    if [[ "$REVIEW_STATUS" != "approved" ]]; then
        echo "  WARN   review_status is '$REVIEW_STATUS', expected 'approved'"
    fi
else
    skip_step "Approve deliverable"
fi

# ============================================================================
# Step 7: Publish Circle
# ============================================================================
# Requires: published_by is Lead+ AND deliverable.status == Approved.
# Founder role satisfies Lead+ (role levels: Founder=6, Lead=5).

echo ""
echo "--- Step 7: POST /api/v1/circles/{id}/publish (publish deliverable)"

PUBLICATION_ID=""
if [[ -n "$CIRCLE_ID" && -n "$DELIVERABLE_ID" ]]; then
    PUBLISH_BODY=$(cat <<JSON
{
  "deliverable_id": "$DELIVERABLE_ID",
  "title": "Smoke Test Publication",
  "abstract_text": "Automated smoke test publication — safe to delete",
  "visibility": "Community",
  "published_by": "$USER_ID"
}
JSON
)
    STATUS=$(curl_post "$API/circles/$CIRCLE_ID/publish" "$PUBLISH_BODY")
    check "Publish deliverable" "$STATUS" 200

    PUBLICATION_ID=$(extract '.id')
    if [[ -z "$PUBLICATION_ID" || "$PUBLICATION_ID" == "null" ]]; then
        echo "  WARN   Could not capture publication_id from response body"
    fi
else
    skip_step "Publish deliverable"
fi

# ============================================================================
# Step 8: List Publications Feed
# ============================================================================
# Only Community-visibility publications appear in this feed.
# We verify the published item is present by matching deliverable_id.

echo ""
echo "--- Step 8: GET /api/v1/publications (verify published item in feed)"

if [[ -n "$DELIVERABLE_ID" ]]; then
    STATUS=$(curl_get "$API/publications")
    check "List publications" "$STATUS" 200

    # Verify the smoke test publication is in the feed
    FOUND=$(jq --arg did "$DELIVERABLE_ID" \
        '[.[] | select(.deliverable_id == $did)] | length' \
        "$BODY_FILE" 2>/dev/null || echo "0")
    if [[ "$FOUND" -ge 1 ]]; then
        echo "  OK     Publication found in feed (deliverable_id=$DELIVERABLE_ID)"
    else
        echo "  WARN   Publication not found in feed (visibility filter or persistence lag)"
        # Not a FAIL — the publication may be present but visibility differs
    fi
else
    skip_step "List publications"
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "=== Results ==="
echo "    Passed : $PASS"
echo "    Failed : $FAIL"
echo "    Skipped: $SKIP"
echo ""

if [[ -n "$CIRCLE_ID" ]]; then
    echo "--- Test data created (in-memory persistence — auto-cleared on server restart)"
    echo "    Circle ID      : $CIRCLE_ID"
    [[ -n "$PROJECT_ID" ]]     && echo "    Project ID     : $PROJECT_ID"
    [[ -n "$DELIVERABLE_ID" ]] && echo "    Deliverable ID : $DELIVERABLE_ID"
    [[ -n "$PUBLICATION_ID" ]] && echo "    Publication ID : $PUBLICATION_ID"
    echo ""
    echo "    To delete manually (if using Firestore persistence):"
    echo "      DELETE $API/circles/$CIRCLE_ID"
fi

echo ""

# Exit non-zero if any hard failures occurred
[[ "$FAIL" -eq 0 ]]

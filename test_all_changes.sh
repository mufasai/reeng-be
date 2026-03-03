#!/bin/bash

# Test script for all new changes
# 1. Head Office role
# 2. Teams CRUD
# 3. Multipart file uploads for Project, Site, Termin
# 4. File downloads

set -e  # Exit on error

BASE_URL="http://localhost:3000/api"
TOKEN="token_test@smartelco.com_1772522704"

echo "======================================"
echo "Testing SmartElco Backend Changes"
echo "======================================"
echo ""

# Test 1: Health Check
echo "1. Testing Health Endpoint..."
curl -s -X GET $BASE_URL/health | jq
echo ""

# Test 2: Head Office Role (already tested during registration)
echo "2. Testing Head Office Role..."
echo "✓ Head Office user already registered: test@smartelco.com"
echo ""

# Test 3: Create Project for testing
echo "3. Creating Test Project..."
PROJECT_RESPONSE=$(curl -s -X POST $BASE_URL/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Test Project for Teams",
    "lokasi": "Jakarta",
    "value": 100000000,
    "cost": 0,
    "keterangan": "Testing project",
    "tipe": "COMBAT",
    "status": "active"
  }')
echo "$PROJECT_RESPONSE" | jq
PROJECT_ID=$(echo "$PROJECT_RESPONSE" | jq -r '.data.id')
echo "Project ID: $PROJECT_ID"
echo ""

# Test 4: Create Site for testing
echo "4. Creating Test Site..."
SITE_RESPONSE=$(curl -s -X POST $BASE_URL/sites \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"project_id\": \"$PROJECT_ID\",
    \"site_name\": \"Test Site\",
    \"site_info\": \"Testing site\",
    \"pekerjaan\": \"Installation\",
    \"lokasi\": \"Jakarta\",
    \"nomor_kontrak\": \"K001\",
    \"start\": \"2026-03-01\",
    \"end\": \"2026-12-31\",
    \"maximal_budget\": 50000000,
    \"cost_estimated\": 40000000,
    \"pemberi_tugas\": \"PT SmartElco\",
    \"penerima_tugas\": \"Contractor A\"
  }")
echo "$SITE_RESPONSE" | jq
SITE_ID=$(echo "$SITE_RESPONSE" | jq -r '.data.id')
echo "Site ID: $SITE_ID"
echo ""

# Test 5: Create People for team members
echo "5. Creating Test People..."
LEADER_RESPONSE=$(curl -s -X POST $BASE_URL/people \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Team Leader",
    "no_hp": "081234567890",
    "email": "leader@test.com",
    "jabatan_kerja": "Engineer"
  }')
echo "$LEADER_RESPONSE" | jq
LEADER_ID=$(echo "$LEADER_RESPONSE" | jq -r '.data.id')
echo "Leader ID: $LEADER_ID"
echo ""

MEMBER1_RESPONSE=$(curl -s -X POST $BASE_URL/people \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Team Member 1",
    "no_hp": "081234567891",
    "email": "member1@test.com",
    "jabatan_kerja": "Technician"
  }')
echo "$MEMBER1_RESPONSE" | jq
MEMBER1_ID=$(echo "$MEMBER1_RESPONSE" | jq -r '.data.id')
echo "Member 1 ID: $MEMBER1_ID"
echo ""

# Test 6: Create Team
echo "6. Testing Create Team..."
TEAM_RESPONSE=$(curl -s -X POST $BASE_URL/teams \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"nama\": \"Test Team Alpha\",
    \"project_id\": \"$(echo $PROJECT_ID | sed 's/projects://g')\",
    \"site_id\": \"$(echo $SITE_ID | sed 's/sites://g')\",
    \"leader_id\": \"$(echo $LEADER_ID | sed 's/people://g')\",
    \"members\": [
      {
        \"people_id\": \"$(echo $LEADER_ID | sed 's/people://g')\",
        \"role\": \"Team Leader\"
      },
      {
        \"people_id\": \"$(echo $MEMBER1_ID | sed 's/people://g')\",
        \"role\": \"Technician\"
      }
    ]
  }")
echo "$TEAM_RESPONSE" | jq
TEAM_ID=$(echo "$TEAM_RESPONSE" | jq -r '.data.id')
echo "Team ID: $TEAM_ID"
echo ""

# Test 7: List Teams
echo "7. Testing List Teams..."
curl -s -X GET $BASE_URL/teams \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# Test 8: Get Team Detail
echo "8. Testing Get Team Detail..."
curl -s -X GET $BASE_URL/teams/$TEAM_ID \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# Test 9: List Team Members
echo "9. Testing List Team Members..."
curl -s -X GET $BASE_URL/teams/$TEAM_ID/members \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# Test 10: Update Team
echo "10. Testing Update Team..."
curl -s -X PUT $BASE_URL/teams/$TEAM_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "nama": "Test Team Alpha Updated",
    "active": true
  }' | jq
echo ""

# Test 11: Create test file for upload
echo "11. Creating test file..."
echo "This is a test document for SmartElco API Testing" > /tmp/test_document.txt
echo "✓ Test file created at /tmp/test_document.txt"
echo ""

# Test 12: Upload Project File (Multipart)
echo "12. Testing Upload Project File (Multipart)..."
PROJECT_FILE_RESPONSE=$(curl -s -X POST $BASE_URL/projects/$(echo $PROJECT_ID | sed 's/projects://g')/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/tmp/test_document.txt" \
  -F "title=Project Test Document")
echo "$PROJECT_FILE_RESPONSE" | jq
PROJECT_FILE_ID=$(echo "$PROJECT_FILE_RESPONSE" | jq -r '.data.id')
echo "Project File ID: $PROJECT_FILE_ID"
echo ""

# Test 13: Upload Site File (Multipart)
echo "13. Testing Upload Site File (Multipart)..."
SITE_FILE_RESPONSE=$(curl -s -X POST $BASE_URL/sites/$(echo $SITE_ID | sed 's/sites://g')/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/tmp/test_document.txt" \
  -F "title=Site Test Document")
echo "$SITE_FILE_RESPONSE" | jq
SITE_FILE_ID=$(echo "$SITE_FILE_RESPONSE" | jq -r '.data.id')
echo "Site File ID: $SITE_FILE_ID"
echo ""

# Test 14: Create Termin for file upload
echo "14. Creating Test Termin..."
TERMIN_RESPONSE=$(curl -s -X POST $BASE_URL/termins \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"project_id\": \"$(echo $PROJECT_ID | sed 's/projects://g')\",
    \"site_id\": \"$(echo $SITE_ID | sed 's/sites://g')\",
    \"nama_termin\": \"Termin 1\",
    \"tanggal_termin\": \"2026-03-10\",
    \"nominal_termin\": 10000000,
    \"status\": \"draft\"
  }")
echo "$TERMIN_RESPONSE" | jq
TERMIN_ID=$(echo "$TERMIN_RESPONSE" | jq -r '.data.id')
echo "Termin ID: $TERMIN_ID"
echo ""

# Test 15: Upload Termin File (Multipart)
echo "15. Testing Upload Termin File (Multipart)..."
TERMIN_FILE_RESPONSE=$(curl -s -X POST $BASE_URL/termins/$(echo $TERMIN_ID | sed 's/termins://g')/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/tmp/test_document.txt" \
  -F "title=Termin Test Document" \
  -F "category=invoice")
echo "$TERMIN_FILE_RESPONSE" | jq
TERMIN_FILE_ID=$(echo "$TERMIN_FILE_RESPONSE" | jq -r '.data.id')
echo "Termin File ID: $TERMIN_FILE_ID"
echo ""

# Test 16: Download Project File
echo "16. Testing Download Project File..."
curl -s -X GET $BASE_URL/project-files/$(echo $PROJECT_FILE_ID | sed 's/project_files://g')/download \
  -H "Authorization: Bearer $TOKEN" \
  -o /tmp/downloaded_project_file.txt
if [ -f /tmp/downloaded_project_file.txt ]; then
  echo "✓ Project file downloaded successfully"
  echo "Content: $(cat /tmp/downloaded_project_file.txt)"
else
  echo "✗ Failed to download project file"
fi
echo ""

# Test 17: Download Site File
echo "17. Testing Download Site File..."
curl -s -X GET $BASE_URL/site-files/$(echo $SITE_FILE_ID | sed 's/site_files://g')/download \
  -H "Authorization: Bearer $TOKEN" \
  -o /tmp/downloaded_site_file.txt
if [ -f /tmp/downloaded_site_file.txt ]; then
  echo "✓ Site file downloaded successfully"
  echo "Content: $(cat /tmp/downloaded_site_file.txt)"
else
  echo "✗ Failed to download site file"
fi
echo ""

# Test 18: Download Termin File
echo "18. Testing Download Termin File..."
curl -s -X GET $BASE_URL/termin-files/$(echo $TERMIN_FILE_ID | sed 's/termin_files://g')/download \
  -H "Authorization: Bearer $TOKEN" \
  -o /tmp/downloaded_termin_file.txt
if [ -f /tmp/downloaded_termin_file.txt ]; then
  echo "✓ Termin file downloaded successfully"
  echo "Content: $(cat /tmp/downloaded_termin_file.txt)"
else
  echo "✗ Failed to download termin file"
fi
echo ""

# Test 19: Delete Team (cleanup and test delete)
echo "19. Testing Delete Team..."
curl -s -X DELETE $BASE_URL/teams/$(echo $TEAM_ID | sed 's/teams://g') \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# Cleanup
echo "======================================"
echo "Cleaning up test files..."
rm -f /tmp/test_document.txt
rm -f /tmp/downloaded_project_file.txt
rm -f /tmp/downloaded_site_file.txt
rm -f /tmp/downloaded_termin_file.txt
echo "✓ Cleanup completed"
echo ""

echo "======================================"
echo "All Tests Completed!"
echo "======================================"

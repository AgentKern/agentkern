# AgentKern Azure Deployment

One-click deployment to Azure Container Apps and Azure AI Foundry.

## Quick Deploy

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fagentkern%2Fagentkern%2Fmain%2F.azure%2Farm-template.json)

## Files

| File | Purpose |
|------|---------|
| `arm-template.json` | Azure Container Apps deployment |
| `ai-foundry-extension.json` | Azure AI Foundry/Hub connection |
| `marketplace-listing.md` | Azure Marketplace content |

## Manual Deployment

### Prerequisites

- Azure CLI installed
- Azure subscription

### Deploy

```bash
# Create resource group
az group create --name agentkern-rg --location eastus

# Deploy ARM template
az deployment group create \
  --resource-group agentkern-rg \
  --template-file arm-template.json \
  --parameters environmentName=prod

# Get gateway URL
az containerapp show \
  --name agentkern-gateway \
  --resource-group agentkern-rg \
  --query properties.configuration.ingress.fqdn
```

### Connect to AI Foundry

```bash
az deployment group create \
  --resource-group agentkern-rg \
  --template-file ai-foundry-extension.json \
  --parameters \
    aiHubName=my-ai-hub \
    agentkernEndpoint=https://agentkern-gateway.azurecontainerapps.io \
    agentkernApiKey=your-api-key
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `AGENTKERN_IDP_API_KEY` | Identity provider API key | For Entra |
| `AGENTKERN_PRODUCTIVITY_API_KEY` | M365 integration | For Teams |
| `AGENTKERN_MODELS_API_KEY` | AI model access | For Nova/Claude |

## Scaling

The ARM template configures:

- Min replicas: 1
- Max replicas: 10
- Scale trigger: 100 concurrent requests

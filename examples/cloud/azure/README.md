# Azure Deployment Guide

Deploy AgentKern to Azure using Container Apps and managed services.

## Prerequisites

- Azure CLI installed and authenticated
- Resource group created
- Container Registry (ACR) available

## Quick Deploy

```bash
# Set variables
export RG_NAME=agentkern-rg
export LOCATION=eastus
export ACR_NAME=agentkernacr
export ENV_NAME=agentkern-env

# Create Container App Environment
az containerapp env create \
  --name $ENV_NAME \
  --resource-group $RG_NAME \
  --location $LOCATION

# Deploy Identity Service
az containerapp create \
  --name agentkern-identity \
  --resource-group $RG_NAME \
  --environment $ENV_NAME \
  --image $ACR_NAME.azurecr.io/agentkern-identity:latest \
  --target-port 3000 \
  --ingress external \
  --min-replicas 2 \
  --max-replicas 10 \
  --cpu 1 \
  --memory 2Gi \
  --env-vars \
    DATABASE_URL=secretref:database-url \
    JWT_SECRET=secretref:jwt-secret \
    NODE_ENV=production
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Azure Resource Group                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────────────────────────┐    │
│  │  Azure      │    │     Container App Environment    │    │
│  │  Front Door │───▶│  ┌─────────────┐ ┌───────────┐  │    │
│  │             │    │  │  Identity   │ │  Pillars  │  │    │
│  └─────────────┘    │  │  Container  │ │  Workers  │  │    │
│                     │  └──────┬──────┘ └─────┬─────┘  │    │
│                     └─────────┼──────────────┼────────┘    │
│                               │              │              │
│  ┌────────────────────────────┼──────────────┼──────────┐  │
│  │                            ▼              ▼           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │  PostgreSQL │  │  Key Vault  │  │  Redis      │   │  │
│  │  │  Flexible   │  │  (secrets)  │  │  Cache      │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  │                    Managed Services                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Managed Services

### PostgreSQL Flexible Server

```bash
az postgres flexible-server create \
  --name agentkern-db \
  --resource-group $RG_NAME \
  --location $LOCATION \
  --admin-user pgadmin \
  --admin-password $(openssl rand -base64 24) \
  --sku-name Standard_B2s \
  --version 15 \
  --storage-size 32 \
  --high-availability Enabled
```

### Key Vault

```bash
az keyvault create \
  --name agentkern-kv \
  --resource-group $RG_NAME \
  --location $LOCATION \
  --enable-rbac-authorization

# Add secrets
az keyvault secret set \
  --vault-name agentkern-kv \
  --name jwt-secret \
  --value $(openssl rand -base64 32)
```

### Redis Cache

```bash
az redis create \
  --name agentkern-redis \
  --resource-group $RG_NAME \
  --location $LOCATION \
  --sku Basic \
  --vm-size C0
```

## Bicep Template

For Infrastructure as Code deployment, use the provided Bicep template:

```bicep
// main.bicep
param location string = resourceGroup().location
param environmentName string = 'agentkern-env'

module containerEnv 'modules/container-env.bicep' = {
  name: 'containerEnv'
  params: {
    name: environmentName
    location: location
  }
}

module identity 'modules/identity-app.bicep' = {
  name: 'identity'
  params: {
    envId: containerEnv.outputs.id
    acrName: 'agentkernacr'
  }
}
```

Deploy with:
```bash
az deployment group create \
  --resource-group $RG_NAME \
  --template-file main.bicep
```

## Scaling

Container Apps auto-scales based on HTTP traffic:

```bash
az containerapp update \
  --name agentkern-identity \
  --resource-group $RG_NAME \
  --scale-rule-name http-scaling \
  --scale-rule-type http \
  --scale-rule-http-concurrency 100
```

## Monitoring

Enable Application Insights:

```bash
az monitor app-insights component create \
  --app agentkern-insights \
  --location $LOCATION \
  --resource-group $RG_NAME \
  --application-type web
```

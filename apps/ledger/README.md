# Ledger Microservice

The secure, ACID-compliant economy ledger service built with Node.js and TypeScript.

## Overview

This microservice manages:
- **Player economy tracking** - Resources, currency, inventory
- **Transaction history** - Immutable ledger of all economic actions
- **ACID guarantees** - Consistency and atomicity for critical operations
- **REST API** - Endpoints for simulation and frontend services

## Technology Stack

- **Node.js 18+** - JavaScript runtime
- **TypeScript** - Type-safe JavaScript
- **Express or Hapi** - REST API framework
- **PostgreSQL 14+** - Transaction storage with ACID guarantees
- **Jest** - Testing framework
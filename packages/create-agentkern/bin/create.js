#!/usr/bin/env node

/**
 * create-agentkern CLI
 * 
 * Scaffolds new AgentKern services with Zero-Trust defaults.
 * 
 * Usage:
 *   npx create-agentkern my-agent
 *   npx create-agentkern my-agent --template rust-service
 */

import { prompts } from 'enquirer';
import chalk from 'chalk';
import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const TEMPLATES = {
  'rust-service': 'A Rust service with mTLS and Gate integration',
  'ts-agent': 'A TypeScript AI Agent with SDK integration',
  'policy-module': 'A WASM policy module for Gate',
};

async function main() {
  console.log(chalk.cyan.bold('\n🚀 create-agentkern\n'));
  console.log(chalk.gray('Scaffold an AgentKern service with Zero-Trust defaults\n'));

  const args = process.argv.slice(2);
  let projectName = args[0];
  let templateName = args.find(a => a.startsWith('--template='))?.split('=')[1];

  // Interactive prompts if not provided
  if (!projectName) {
    const { name } = await prompts({
      type: 'input',
      name: 'name',
      message: 'Project name:',
      initial: 'my-agentkern-service',
    });
    projectName = name;
  }

  if (!templateName) {
    const { template } = await prompts({
      type: 'select',
      name: 'template',
      message: 'Select template:',
      choices: Object.entries(TEMPLATES).map(([name, description]) => ({
        name,
        message: `${name} - ${description}`,
      })),
    });
    templateName = template;
  }

  const targetDir = path.resolve(process.cwd(), projectName);
  const templateDir = path.join(__dirname, '..', 'templates', templateName);

  if (fs.existsSync(targetDir)) {
    console.log(chalk.red(`\n❌ Directory ${projectName} already exists`));
    process.exit(1);
  }

  console.log(chalk.yellow(`\n📁 Creating ${projectName}...`));

  // Copy template
  if (!fs.existsSync(templateDir)) {
    console.log(chalk.red(`\n❌ Template ${templateName} not found`));
    process.exit(1);
  }

  await fs.copy(templateDir, targetDir);

  // Replace placeholders
  const files = await fs.readdir(targetDir, { recursive: true });
  for (const file of files) {
    const filePath = path.join(targetDir, file);
    if ((await fs.stat(filePath)).isFile()) {
      let content = await fs.readFile(filePath, 'utf-8');
      content = content.replace(/{{PROJECT_NAME}}/g, projectName);
      content = content.replace(/{{PROJECT_NAME_SNAKE}}/g, projectName.replace(/-/g, '_'));
      await fs.writeFile(filePath, content);
    }
  }

  console.log(chalk.green('\n✅ Project created successfully!\n'));
  console.log(chalk.white('Next steps:'));
  console.log(chalk.gray(`  cd ${projectName}`));
  
  if (templateName === 'rust-service') {
    console.log(chalk.gray('  cargo build'));
    console.log(chalk.gray('  cargo run'));
  } else {
    console.log(chalk.gray('  npm install'));
    console.log(chalk.gray('  npm run dev'));
  }

  console.log(chalk.cyan('\n📖 Read more: https://github.com/AgentKern/agentkern\n'));
}

main().catch(err => {
  console.error(chalk.red('Error:'), err);
  process.exit(1);
});

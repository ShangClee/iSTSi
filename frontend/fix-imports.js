#!/usr/bin/env node

import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { join } from 'path';

const uiDir = './src/components/ui';
const files = readdirSync(uiDir).filter(file => file.endsWith('.tsx'));

files.forEach(file => {
  const filePath = join(uiDir, file);
  let content = readFileSync(filePath, 'utf8');
  
  // Fix Radix UI imports with version numbers
  content = content.replace(/@radix-ui\/react-([^@]+)@[^"]+/g, '@radix-ui/react-$1');
  
  // Fix lucide-react imports with version numbers
  content = content.replace(/lucide-react@[^"]+/g, 'lucide-react');
  
  // Fix class-variance-authority imports with version numbers
  content = content.replace(/class-variance-authority@[^"]+/g, 'class-variance-authority');
  
  writeFileSync(filePath, content);
  console.log(`Fixed imports in ${file}`);
});

console.log('All imports fixed!');
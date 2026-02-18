const fs = require('node:fs');
const path = require('node:path');
const { parseArgs } = require('node:util');

// --- Configuration ---
const options = {
    address: { type: 'string', short: 'a', default: '127.0.0.1:8899' },
    service: { type: 'string', short: 's', default: 'card' }, 
    number: { type: 'string', short: 'n', default: '5' },   
    repeats: { type: 'string', short: 'r', default: '1000' },
    path: { type: 'string', short: 'p', default: '../asset/public/idol' } 
};

const { values } = parseArgs({ args: process.argv.slice(2), options, strict: false });

const CONFIG = {
    address: values.address,
    service: values.service,
    number: parseInt(values.number),
    repeats: parseInt(values.repeats),
    assetsPath: path.resolve(process.cwd(), values.path)
};

// --- Helpers ---

// Load IDs from DIRECTORIES
let dirContent = [];
try {
    if (fs.existsSync(CONFIG.assetsPath)) {
        const entries = fs.readdirSync(CONFIG.assetsPath, { withFileTypes: true });
        dirContent = entries
            .filter(entry => entry.isDirectory()) 
            .map(entry => parseInt(entry.name))   
            .filter(id => !isNaN(id));            
    }
} catch (e) {
    console.error(`Warning: Could not read directory ${CONFIG.assetsPath}`);
}

if (dirContent.length === 0) {
    console.error(`\n[ERROR] No ID folders found in: ${CONFIG.assetsPath}`);
    process.exit(1);
}

function getRandomInt(min, max) {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

function getRandomCard() {
    const frameType = getRandomInt(0, 2);

    return {
        id: dirContent[Math.floor(Math.random() * dirContent.length)],
        variant: 1,      
        frame_type: frameType
    };
}

function getRandomBatch() {
    const batch = [];
    for (let i = 0; i < CONFIG.number; i++) {
        batch.push(getRandomCard());
    }
    return { cards: batch };
}

function generateHash(payload) {
    const jsonStr = JSON.stringify(payload);
    return Buffer.from(jsonStr).toString('base64url');
}

async function getRenderTime(service, hash) {
    const url = `http://${CONFIG.address}/render/${service}/${hash}`;
    
    try {
        const response = await fetch(url);
        
        if (!response.ok) {
            const text = await response.text();
            console.error(`\n[FAIL] Payload rejected: ${JSON.stringify(JSON.parse(Buffer.from(hash, 'base64url').toString()))}`);
            throw new Error(`HTTP ${response.status}: ${text}`);
        }

        const headerVal = response.headers.get('x-processing-time');
        if (!headerVal) throw new Error("Missing 'x-processing-time' header");

        return parseFloat(headerVal);
    } catch (err) {
        console.error(`[ERROR] ${err.message}`);
        process.exit(1);
    }
}

// --- Benchmark ---

async function benchService(serviceName, payloadGenerator) {
    let total = 0;
    for (let i = 0; i < CONFIG.repeats; i++) {
        const hash = generateHash(payloadGenerator());
        total += await getRenderTime(serviceName, hash);
        if (i % 100 === 0) process.stdout.write('.'); 
    }
    process.stdout.write('\n');
    return total;
}

async function main() {
    console.log(`Starting benchmark against ${CONFIG.address}...`);
    console.log(`Assets loaded: ${dirContent.length} IDs`);

    const servicesToRun = CONFIG.service === 'all' ? ['card', 'fan', 'album'] : [CONFIG.service];

    for (const s of servicesToRun) {
        console.log(`Benchmarking ${s} renders (${CONFIG.repeats} repeats)...`);
        const totalTime = await benchService(s === 'card' ? 'card' : s, s === 'card' ? getRandomCard : getRandomBatch);
        console.log(`Average ${s} render time: ${(totalTime / CONFIG.repeats).toFixed(4)} ms`);
        console.log('-'.repeat(40));
    }
    console.log("Done");
}

main();
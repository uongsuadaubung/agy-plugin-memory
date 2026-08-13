import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { unlinkSync, existsSync } from "fs";
import { join } from "path";
import {
  getOrCreateProject,
  addMemory,
  getMemories,
  searchMemories,
  toggleMemoryPermanence,
  deleteMemory,
  listProjects,
  cleanupExpired
} from "../src/db";

const TEST_DIR = process.cwd();
const TEST_DB = join(TEST_DIR, ".antigravity_memory.db");

describe("Project Memory MCP Server (Permanent vs Short-term)", () => {
  beforeAll(() => {
    if (existsSync(TEST_DB)) {
      try { unlinkSync(TEST_DB); } catch (_) {}
    }
  });

  afterAll(() => {
    if (existsSync(TEST_DB)) {
      try { unlinkSync(TEST_DB); } catch (_) {}
    }
  });

  test("auto-detect and create project", () => {
    const project = getOrCreateProject(undefined, TEST_DIR);
    expect(project).toBeDefined();
    expect(project.id).toBeString();
  });

  test("add short-term and permanent memories", () => {
    const project = getOrCreateProject(undefined, TEST_DIR);
    
    // Short-term memory
    const shortMem = addMemory(
      project.id,
      "Fixed typo in navbar component",
      ["bugfix", "ui"],
      {},
      false,
      TEST_DIR
    );
    expect(shortMem.is_permanent).toBeFalse();

    // Permanent memory
    const permMem = addMemory(
      project.id,
      "Core Rule: Always use Bun runtime and bun:sqlite for DB storage",
      ["architecture", "rule"],
      {},
      true,
      TEST_DIR
    );
    expect(permMem.is_permanent).toBeTrue();
  });

  test("filter memories by permanence", () => {
    const project = getOrCreateProject(undefined, TEST_DIR);
    
    const permOnly = getMemories(project.id, 10, undefined, true, TEST_DIR);
    expect(permOnly.length).toBeGreaterThan(0);
    expect(permOnly.every((m) => m.is_permanent)).toBeTrue();

    const shortOnly = getMemories(project.id, 10, undefined, false, TEST_DIR);
    expect(shortOnly.length).toBeGreaterThan(0);
    expect(shortOnly.every((m) => !m.is_permanent)).toBeTrue();
  });

  test("toggle memory permanence", () => {
    const project = getOrCreateProject(undefined, TEST_DIR);
    const memories = getMemories(project.id, 10, undefined, false, TEST_DIR);
    expect(memories.length).toBeGreaterThan(0);

    const targetId = memories[0].id;
    const updated = toggleMemoryPermanence(targetId, true, TEST_DIR);
    expect(updated).not.toBeNull();
    expect(updated?.is_permanent).toBeTrue();
  });

  test("cleanup preserves permanent memories", () => {
    const project = getOrCreateProject(undefined, TEST_DIR);
    
    // Add 25 short term memories to trigger rolling limit (max 20)
    for (let i = 0; i < 25; i++) {
      addMemory(project.id, `Temp memory ${i}`, ["temp"], {}, false, TEST_DIR);
    }

    const permBefore = getMemories(project.id, 10, undefined, true, TEST_DIR);
    const countPermBefore = permBefore.length;

    cleanupExpired(project.id, 5, 0, TEST_DIR);

    const permAfter = getMemories(project.id, 10, undefined, true, TEST_DIR);
    expect(permAfter.length).toBe(countPermBefore);
  });
});

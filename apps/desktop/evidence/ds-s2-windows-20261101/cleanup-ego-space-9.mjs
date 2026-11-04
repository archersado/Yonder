const task = await taskSpace(9);
await task.finish({ keep: [] });
console.log(JSON.stringify({ spaceId: task.spaceId, finished: true }));

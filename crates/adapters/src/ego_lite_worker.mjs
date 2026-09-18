const reply = value => fs.writeFile(responsePath, JSON.stringify({ operation: request.operation, ...value }) + '\n');
const state = async task => ({
  external_task_ref: `ego:${task.spaceId}`,
  ownership: String(task.ownership),
  managed_pages: (await task.pages()).length,
});

try {
  if (request.operation === 'create') {
    const task = await taskSpace(request.name);
    await reply(await state(task));
  } else {
    const spaceId = Number(String(request.external_task_ref).replace(/^ego:/, ''));
    if (!Number.isSafeInteger(spaceId) || spaceId <= 0) throw new Error('invalid reference');
    if (request.operation === 'take_over') {
      const task = await takeOverTaskSpace(spaceId);
      await reply(await state(task));
    } else {
      const task = await taskSpace(spaceId);
      if (request.operation === 'observe') await reply(await state(task));
      else if (request.operation === 'hand_off') {
        const managed_pages = (await task.pages()).length;
        await task.handOff();
        const observed = (await listTaskSpaces()).find(space => space.id === spaceId);
        if (!observed) throw new Error('space missing after handoff');
        await reply({ external_task_ref: `ego:${spaceId}`, ownership: String(observed.ownership), managed_pages });
      }
      else if (request.operation === 'finish') { await task.finish({ keep: [] }); await reply({ external_task_ref: `ego:${spaceId}`, finished: true }); }
      else throw new Error('invalid operation');
    }
  }
} catch {
  process.exit(3);
}

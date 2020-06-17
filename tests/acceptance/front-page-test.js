import { currentURL, visit } from '@ember/test-helpers';
import { setupApplicationTest } from 'ember-qunit';
import { module, test } from 'qunit';

import a11yAudit from 'ember-a11y-testing/test-support/audit';
import { percySnapshot } from 'ember-percy';

import axeConfig from '../axe-config';
import { title } from '../helpers/dom';
import setupMirage from '../helpers/setup-mirage';

module('Acceptance | front page', function (hooks) {
  setupApplicationTest(hooks);
  setupMirage(hooks);

  test('is accessible', async function (assert) {
    assert.expect(0);

    this.server.loadFixtures();

    await visit('/');
    await a11yAudit(axeConfig);
  });

  test('visiting /', async function (assert) {
    this.server.loadFixtures();

    await visit('/');

    assert.equal(currentURL(), '/');
    assert.equal(title(), 'crates.io: Rust Package Registry');

    assert.dom('[data-test-install-cargo-link]').exists();
    assert.dom('[data-test-all-crates-link]').exists();
    assert.dom('[data-test-login-link]').exists();

    assert.dom('[data-test-total-downloads] [data-test-value]').hasText('143,345');
    assert.dom('[data-test-total-crates] [data-test-value]').hasText('23');

    assert.dom('[data-test-new-crates] [data-test-crate-link="0"]').hasText('Inflector v1.0.0');
    assert.dom('[data-test-new-crates] [data-test-crate-link="0"]').hasAttribute('href', '/crates/Inflector');

    assert.dom('[data-test-most-downloaded] [data-test-crate-link="0"]').hasText('serde');
    assert.dom('[data-test-most-downloaded] [data-test-crate-link="0"]').hasAttribute('href', '/crates/serde');

    assert.dom('[data-test-just-updated] [data-test-crate-link="0"]').hasText('nanomsg v0.6.1');
    assert.dom('[data-test-just-updated] [data-test-crate-link="0"]').hasAttribute('href', '/crates/nanomsg');

    percySnapshot(assert);
  });
});

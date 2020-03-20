Wok -- control several git repositories as a single project
===========================================================

.. toctree::
   :maxdepth: 2
   :caption: Contents:

Wok is a command line tool which allows you to control several git repositories in a
centralized manner, say as a single workspace.

Wok as easy to use as:

.. code-block:: shell

   # Init the workspace config in `./wok.yml`
   $ wok init
   # Add some repos to the workspace
   $ wok add git@github.com:user/sub-repo-1.git ./sub-repo-1
   $ wok add git@github.com:user/sub-repo-2.git ./sub-repo-2
   # Start work on a new feature
   $ wok start new-feature
   # Join a repo into the new feature's scope
   $ wok join sub-repo-2
   # ... make changes to files and commit everything in this and sub repos
   # Push your changes into a remote feature branches of each repo
   $ wok push
   # Merge feature into the main branches in all repos
   $ wok finish
   # Tag a release in all repos
   $ wok tag v1.0.0

Features
--------

- Manage current state of a group of repositories -- a workspace
- Push the entire workspace at once
- Create feature branches in affected repos only
- Branch out the entire workspace at once
- Merge feature branches using one of the strategies:
   - merge with merge commit
   - rebase, merge with merge commit
   - rebase, squash, merge fast-forward only
- Tag releases in all branches at once and push new tags 

Installation
------------

Install Wok by running:

.. code-block:: shell

   pip install wok

Contribute
----------

- Issue Tracker: github.com/lig/wok/issues
- Source Code: github.com/lig/wok

Support
-------

If you are having issues, please let us know.
The best way is to report an issue here: https://github.com/lig/wok/issues/new

License
-------

The project is licensed under the MIT license.

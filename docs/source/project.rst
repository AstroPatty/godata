Projects API
============

The godata projects API is the primary tool you will use to interact with your project.
It provides tools for creating and loading projects, as well as adding, getting and
removing project data. If you're using godata in a script, Jupyter notebook or even
a larger application, you probably won't ever have to go beyond the tools documented
here.

======================
Projects & Collections
======================

Godata projects can be organized into collections, which can be used to group reltated
projects together. Collections are automatically create when you create a project inside
a collection, and are automatically deleted when the last project in a collection is
deleted. Creating a project inside a collection is as simple as:

.. code-block:: python

    from godata import create_project

    # Create 'my_project' inside 'my_collection'
    project = create_project('my_project', collection='my_collection')

If you do not specify a collection, the project will be created in the 'default'
collection.

Loading projects that you have previously created is just as easy:

.. code-block:: python

    from godata import load_project

    # Load 'my_project' from 'my_collection'
    project = load_project('my_project', collection='my_collection')

Projects can only be created once, but can be loaded any number of times from any
number of python sessions including those executing concurrently.

If you forget the name of a project or collection, you can list them using the
``list_projects`` and ``list_collections`` functions:

.. code-block:: python

    from godata import list_projects, list_collections

    # List all collections
    print(list_collections())

    # List all projects in 'my_collection'
    print(list_projects("my_collection"))

Output:

.. code-block:: output

    ['default', 'my_collection']
    ['my_project']

-------------------------------
Hidden Collections and Projects
-------------------------------

Hidden collections and projects are hidden from the ``list_collections`` and ``list_projects``
functions by default. You can list hidden collections and projects by passing the
``hidden=True`` argument to the list functions. In general, hidden collections and projects
are designed to be used by libraries built on top of godata, which store data
that is not intended to be accessed directly by the user. Note that projects and collections
are hidden seperately. You can place a hidden project inside a visible collection, or a
visible project inside a hidden collection.

Creating a hidden project is as simple as prepending the project name with a period:

.. code-block:: python

    from godata import create_project, list_projects

    # Create a hidden project
    project = create_project('.my_hidden_project', collection='my_collection')

    # List all projects in 'my_collection'
    print(list_projects("my_collection"))

    # List all projects in 'my_collection', including hidden projects
    print(list_projects("my_collection", hidden=True))

Output:

.. code-block:: output

    ['my_project']
    ['.my_hidden_project', 'my_project']

======================
Working with Projects
======================

Once you have created or loaded a project, working with it is simple.
The :class:`godata.project.GodataProject` class provides a number of methods for
adding, getting, listing, and removing data from the project. In general, you should
never create a :class:`godata.project.GodataProject` object directly. You should always
use the ``create_project`` and ``load_project`` functions.

-------------------------
Project Paths and Folders
-------------------------

Within a project, all data is stored and accessed using a *project path*. A project path
looks a lot like a file path, but it is always relative to the project root. For example,
if you have a project with the following structure:

.. code-block:: output

    my_project/
        data/
            file1.txt
            file2.txt
        results/
            file3.txt
            file4.txt
            more_results/
                file5.txt

The project path for ``file1.txt`` would be ``data/file1.txt``, the project path for
``file3.txt`` would be ``results/file3.txt``, and the project path for ``file5.txt`` \
would be ``results/more_results/file_five.txt``. File extensions are not required, and you
could just as easily use ``data/file1``, ``results/file3``, and ``results/more_results/file5``.

Everything prior to the last slash in a project path is considered a *folder*. Folders
are automatically created when you add data to a project, and are automatically deleted
when the last file in a folder is removed. Your folders can contain subfolders and files,
just like a regular file system.

========================
Linking vs. Storing Data
========================

When you add data to a project, you can choose to link to the data or store it in the project.
When it comes to using the data, there is no difference between linked and stored data.

:meth:`GodataProject.link <godata.project.GodataProject.link>`  is used to link pre-existing
data to a project without actually copying the data into godata. This is useful when you have
a large amount of pre-existing data that you want to work with in godata. If the
data is in a format that godata can read, it will be returned as a python object when you
request it.

One key difference between linked and stored data is that linked data does not have to
be a known format. If you link to a file that godata does not recognize, it will always
be returned as a path when you request it.

A second key difference is the way that linked and stored data will be handled when you
delete them from a project (or delete the project itself). Linked data will *never* be
deleted by godata under any circumstances.

:meth:`GodataProject.store <godata.project.GodataProject.store>` is used to store data in
godata. This is most useful when you are creating and storing objects in python that
you want to able be access later. Godata will handle the serialization and deserialization
of the data for you, and track it on disk.

Unlike linked data, stored data *will* be deleted by godata when you delete it from a project
or when the project itself is deleted.

You can also store data that already exists on disk by passing a path to the ``store`` method.
In this case, godata will create a copy of the data, while leaving the original data untouched.
If you delete the stored data, the copy will be deleted, but the original data will be left
on disk in its original location.




=============
API Reference
=============

The :meth:`godata.create_project` and :meth:`godata.load_project` return a :class:`godata.project.GodataProject`
object, which is used to interact with the project. You should never create a :class:`godata.project.GodataProject`
object directly.


.. autofunction:: godata.create_project
.. autofunction:: godata.load_project
.. autofunction:: godata.delete_project

.. autofunction:: godata.has_project
.. autofunction:: godata.has_collection
.. autofunction:: godata.list_projects
.. autofunction:: godata.list_collections

.. autoclass:: godata.project.GodataProject
    :members: link, store, get, move, remove, list, ls, has_path

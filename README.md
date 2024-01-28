## godata: data management for the working scientist

godata is a full-service data management tool for scientists working with data in Python. It allows you to forget about files and think more about your science. godata is suited to interactive computing (such as notebooks) or scripts producing hundreds or even thousands of outputs. It is generally thread-safe, and changes are immediately available across all Python interpreters on your system.

### Quickstart

godata is available for Python >= 3.10 on Linux and Mac. 

godata can be easily installed with pip:

```bash
>> pip install godata
```

The godata server tracks the location of all data in your projects, and runs outside your python interpreter. Install it with: 

```bash
>> godata server install
```

From a python terminal, let's create a new project named 'My-project' in collection 'My-collection'

```
>> from godata import create_project
>> project = create_project("My-project", collection="My-collection")
Project My-project created in collection My-collection
```

In godata, _projects_ hold files and folders while _collections_ group related projects together.

Let's create some pretend data and store it in the project.

```
>> import numpy as np
>> data = np.random.rand(1000, 1000)
>> project.store(data, "data/my-important-data")
True
```
This data is now available wherever godata is installed. From a separte python terminal:

```
>> from godata import load_project
>> project = load_project("My-project", collection="My-collection")
Sucessfully loaded project My-collection/My-project

>> data = project.get("data/my-important-data")
>> data
array([[0.17953348, 0.46537863, 0.05551358, ..., 0.3526392 , 0.88688146,
        0.03827608],
       [0.10724396, 0.6594922 , 0.5464358 , ..., 0.25741847, 0.69896045,
        0.39693609],
       [0.13837611, 0.26225224, 0.17050776, ..., 0.07945242, 0.25077166,
        0.9795102 ],
       ...,
       [0.44973845, 0.30379528, 0.90120586, ..., 0.55909527, 0.72301093,
        0.52067499],
       [0.60742971, 0.41941807, 0.82986818, ..., 0.57479954, 0.04413556,
        0.6444287 ],
       [0.10467444, 0.52236499, 0.54443629, ..., 0.35378595, 0.07125344,
        0.59053222]])
```
godata is not a substitute for good organization. You should still try to give your files and folders informative names. But godata handles all output and input of python objects, and tracks where files are so you don't have to remember.

If you've ever forgotten what's in a given folder, it's easy to check:

```
>> project.ls()
Project `My-project` root:
--------------------------
  data/

>> project.ls("data")
My-project/data:
----------------
  my-important-data
```
For more programattic access, you can also get lists of the files and folders at a given path with ```project.list()```. You can remove a file from your project just as easily

```
>> project.remove("data/my-important-data")
True

>> project.list()
{'folders': [], 'files': []}
```

godata doesn't want you to have to think about creating or deleting folders. The folder 'data' was automatically deleted when the last file was removed from it. You may have also noticed it was also automatically created when you put an object in it.



import math
import time

from pyemc.abstractions.test_api import TestCase
from pyemc.client import DeleteFailedError
from pyemc.client import ItemNotFoundError
from pyemc.client import SetFailedError
from pyemc.util import generate_random_data
from pyemc.util import generate_random_key


class TestApi(TestCase):

    ## Happy path

    def test_set_and_get_small_key(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        self.write("Setting small key:   %r -> %r" % (key, val))
        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value
        self.write("Retrieved small key: %r -> %r" % (key, val2))

        assert val == val2

    def test_set_and_get_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 19)  # .5mb

        self.write("Setting large value (%s):   %r -> %r..." % (len(val), key, val[:7]))
        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value
        self.write("Retrieved large value (%s): %r -> %r..." % (len(val), key, val2[:7]))

        assert val == val2

    def test_get_multiple(self):
        key1 = generate_random_key(10)
        val1 = generate_random_data(10)

        key2 = generate_random_key(10)

        key3 = generate_random_key(10)
        val3 = generate_random_data(10)

        self.write("Setting key %r -> %r" % (key1, val1))
        self.client.set(key1, val1)

        self.write("Setting key %r -> %r" % (key3, val3))
        self.client.set(key3, val3)

        keys = [key1, key2, key3]
        self.write("Getting keys %r" % keys)
        dct = self.client.get_multi(keys)
        self.write("Got keys: %r" % dct.keys())

        assert val1 == dct[key1].value
        assert val3 == dct[key3].value

    def test_set_exptime_abs_1s(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        self.client.set(key, val, exptime=int(math.floor(time.time()) + 1))
        item = self.client.get(key)  # still there

        time.sleep(1.1)
        with self.assert_raises(ItemNotFoundError):
            item = self.client.get(key)  # expired

    def test_set_exptime_rel_1s(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        self.client.set(key, val, exptime=1)
        item = self.client.get(key)  # still there

        time.sleep(1.1)
        with self.assert_raises(ItemNotFoundError):
            item = self.client.get(key)  # expired

    def test_set_flags(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)
        flags = 15

        self.client.set(key, val, flags=flags)
        item = self.client.get(key)

        flags2 = item.flags
        val2 = item.value

        assert val == val2
        assert flags == flags2

    def test_set_noreply(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        # set without requesting confirmation
        self.client.set(key, val, noreply=True)

        # verify that it worked
        item = self.client.get(key)
        val2 = item.value

        assert val == val2

    def test_delete(self):
        key = generate_random_key(8)
        val = generate_random_data(5, 8)

        self.client.set(key, val)
        self.client.delete(key)
        with self.assert_raises(ItemNotFoundError):
            self.client.get(key)

        key = generate_random_key(8)
        with self.assert_raises(DeleteFailedError):
            self.client.delete(key)

    def test_delete_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(5, 8)

        self.client.set(key, val)
        self.client.delete(key, noreply=True)
        with self.assert_raises(ItemNotFoundError):
            self.client.get(key)


    ## Failure cases

    def test_get_invalid_key(self):
        key = generate_random_key(4)

        self.write("Trying to get invalid key...")
        with self.assert_raises(ItemNotFoundError):
            item = self.client.get(key)

        self.write("...key not found")


    ## Exceed limits

    def test_set_too_large_key(self):
        key = generate_random_key(251)  # limit is 250b
        val = generate_random_data(1)

        self.write("Trying to set too large key (%s):   %r -> %r..." % (len(key), key[:7], val))
        with self.assert_raises(SetFailedError):
            self.client.set(key, val)

        self.write("...set failed")

    def test_set_too_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 21)  # 2mb, limit is 1mb

        self.write("Trying to set too large value (%s):   %r -> %r..." % (len(val), key, val[:7]))
        with self.assert_raises(SetFailedError):
            self.client.set(key, val)

        self.write("...set failed")
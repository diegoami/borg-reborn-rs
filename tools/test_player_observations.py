"""Known-view rendering, native log boundaries and strict historical extensions."""
import json
from pathlib import Path
import tempfile
import unittest

import borg_runtime as runtime
import borg_replay_viewer as viewer


def frame(index, turn=10, full=False):
    return {'index':index,'turn':turn,'phase':'key','width':80,'height':2,'full':full,
            'hp':4,'max_hp':15,'sp':1,'max_sp':4,'level':2,'depth':1,'x':1,'y':1,
            'cells':[[0,64,1]] if full else [],'palette':None,'details':None}


class OriginalBoundaries(unittest.TestCase):
    def fixture(self, path):
        rows=[frame(1,full=True),frame(2),frame(3,11)]
        (path/'frames.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in rows))
        movie=viewer.Frames(path);self.addCleanup(movie.close)
        return movie

    def check_boundaries(self, log, movie):
        for mode in (0,1):
            text='\n'.join(row[0] for row in log.window(movie.get(0)[0],0,20,100,mode=mode)[0])
            self.assertIn('Casting early',text);self.assertNotIn('Quaffing later',text)
        text='\n'.join(row[0] for row in log.window(movie.get(0)[0],0,20,100,mode=2)[0])
        self.assertNotIn('Game event later',text)
        text='\n'.join(row[0] for row in log.window(movie.get(1)[0],1,20,100,mode=2)[0])
        self.assertIn('Game event later',text)

    def test_headless_keys_hide_later_same_turn_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary);movie=self.fixture(path)
            (path/'messages.log').write_text('0 10 0 # Casting early\n1 10 0 # Quaffing later\n1 10 7 Game event later\n')
            self.check_boundaries(viewer.ReplayLog(path,movie),movie)

    def test_gui_frame_boundaries_hide_later_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary);movie=self.fixture(path)
            rows=[{'frame':0,'turn':10,'text':'# Casting early'},{'frame':1,'turn':10,'text':'# Quaffing later'},{'frame':1,'turn':10,'text':'Game event later'}]
            (path/'native-messages.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in rows))
            self.check_boundaries(viewer.ReplayLog(path,movie),movie)

    def test_only_declared_absent_fields_may_be_added(self):
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary);original=frame(1,full=True);extended={**original,'inventory':{},'spells':{}}
            (path/'original').write_text(json.dumps(original)+'\n');(path/'extended').write_text(json.dumps(extended)+'\n')
            runtime.compare_observation_extension(path/'original',path/'extended')
            for key,value in [('hp',5),('unknown_extra','invented')]:
                changed={**extended,key:value};(path/'extended').write_text(json.dumps(changed)+'\n')
                with self.assertRaisesRegex(ValueError,'original frame diverges'):runtime.compare_observation_extension(path/'original',path/'extended')
            (path/'extended').write_text('')
            with self.assertRaisesRegex(ValueError,'frame count'):runtime.compare_observation_extension(path/'original',path/'extended')

    def test_existing_observation_field_cannot_be_replaced(self):
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary);original={**frame(1,full=True),'spells':{'caster':True}};extended={**original,'spells':{'caster':False}}
            for name,row in [('original',original),('extended',extended)]: (path/name).write_text(json.dumps(row)+'\n')
            with self.assertRaisesRegex(ValueError,'original frame diverges'):runtime.compare_observation_extension(path/'original',path/'extended')


class KnownPanels(unittest.TestCase):
    def test_inventory_does_not_fall_back_to_spoiler_diagnostics(self):
        rows=viewer.inventory_rows({'diagnostics':{'gear':[{'actual_kind':'Secret Potion'}]}})
        text=' '.join(row[0] for row in rows)
        self.assertIn('not captured',text);self.assertNotIn('Secret Potion',text)

    def test_spells_expose_native_cost_failure_book_and_restriction(self):
        state={'spells':{'class':'Necromancer','mana':0,'max_mana':4,'caster':True,'learned':[
            {'name':'Nether Bolt','label':'a','book':'Known Book','book_label':'b','level':1,'mana':1,'fail':36,'castable':True,'status':'Low mana'}]}}
        text=' '.join(row[0] for row in viewer.spell_rows(state))
        for expected in ('Nether Bolt','36%','Low mana','Known Book','overexertion'):self.assertIn(expected,text)
        self.assertIn('no spells',viewer.spell_rows({'caster':False})[1][0])

    def test_sidebar_mismatch_is_not_silently_corrected(self):
        state=frame(1,full=True);text='HP 3/15'.ljust(80)+'SP 0/4'.ljust(80);chars=list(map(ord,text))
        message,pending=viewer.playback_status(state,chars,3)
        self.assertTrue(pending);self.assertIn('awaiting redraw',message);self.assertIn('recorded HP 4/15',message)
        self.assertEqual(chars,list(map(ord,text)))


if __name__=='__main__': unittest.main()
